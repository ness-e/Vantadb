//! `POST /conversation/add` application use case (B1 — cleanCA A2 🔴).
//!
//! `conversation_add` in [`crate::server::handlers`] used to orchestrate
//! `create_thread` + `send_message` + audit + the MEM-55 memory-pipeline
//! trigger inline in the HTTP handler. This module holds that orchestration
//! behind a port trait so the handler stays a humble object
//! (DTO → use case → response) and the flow is unit-testable without HTTP.

use crate::audit::AuditEvent;
use crate::error::Result;
use crate::sdk::Embedded;
use std::sync::Arc;

use super::state::ConversationTrigger;

/// Input for [`StartConversationUseCase::execute`].
///
/// Mirrors [`crate::server::handlers::ConversationAddRequest`] with the wire
/// parsing already done: `thread_id` is a parsed `u128`, `title` keeps its
/// `Option` so the domain default (`"conversation"`) lives here, not in HTTP.
#[derive(Debug, Clone)]
pub struct StartConversationCommand {
    /// Existing thread id. `None` = create a thread first.
    pub thread_id: Option<u128>,
    /// Thread title, used only when creating. Defaults to `"conversation"`.
    pub title: Option<String>,
    /// TTL for a newly created thread.
    pub ttl_secs: Option<u64>,
    /// Message role (`user`, `assistant`, ...).
    pub role: String,
    /// Message content.
    pub content: String,
    /// Caller tracing id (SRV-02) carried into the audit event.
    pub request_id: Option<String>,
}

/// Port abstracting the SDK + trigger surface the use case orchestrates.
///
/// Production: [`ServerConversationPorts`]. Tests: in-memory fake.
pub trait StartConversationPorts {
    /// Create a thread; returns its numeric id.
    fn create_thread(&self, title: &str, ttl_secs: Option<u64>) -> Result<u128>;
    /// Append a message to an existing thread.
    fn send_message(&self, thread_id: u128, role: &str, content: &str) -> Result<()>;
    /// Record the `conversation/threads` audit event (best-effort, never fails).
    fn record_audit(&self, thread_id: u128, request_id: Option<&str>);
    /// Fire the MEM-55 memory-pipeline trigger (best-effort; error is
    /// swallowed by the caller per P4 — extraction failures never fail
    /// the request).
    fn fire_trigger(
        &self,
        thread_id: u128,
        role: &str,
        content: &str,
    ) -> std::result::Result<(), String>;
}

/// Starts (or reuses) a conversation thread and records one message.
///
/// Order of effects (unchanged from the inline handler): create → send →
/// audit → trigger.
pub struct StartConversationUseCase;

impl StartConversationUseCase {
    /// Executes the command against `ports`.
    pub fn execute(
        ports: &impl StartConversationPorts,
        cmd: StartConversationCommand,
    ) -> Result<u128> {
        let title = cmd.title.unwrap_or_else(|| "conversation".to_string());
        let id = match cmd.thread_id {
            Some(id) => id,
            None => ports.create_thread(&title, cmd.ttl_secs)?,
        };
        ports.send_message(id, &cmd.role, &cmd.content)?;
        ports.record_audit(id, cmd.request_id.as_deref());
        // P4: extraction failures never fail the request.
        if let Err(err) = ports.fire_trigger(id, &cmd.role, &cmd.content) {
            tracing::warn!(thread = %id, %err, "conversation trigger failed; ignoring");
        }
        Ok(id)
    }
}

/// Production [`StartConversationPorts`] backed by the [`Embedded`] SDK
/// handle plus the optional server-level conversation trigger.
pub struct ServerConversationPorts<'a> {
    /// Embedded SDK handle (thread store + audit).
    pub db: &'a Embedded,
    /// Optional MEM-55 post-save hook.
    pub trigger: Option<Arc<dyn ConversationTrigger>>,
}

impl StartConversationPorts for ServerConversationPorts<'_> {
    fn create_thread(&self, title: &str, ttl_secs: Option<u64>) -> Result<u128> {
        self.db.create_thread(title, ttl_secs)
    }

    fn send_message(&self, thread_id: u128, role: &str, content: &str) -> Result<()> {
        self.db.send_message(thread_id, role, content)
    }

    fn record_audit(&self, thread_id: u128, request_id: Option<&str>) {
        self.db.audit(
            AuditEvent::memory(
                "conversation",
                "threads",
                &thread_id.to_string(),
                "ok",
                None,
            )
            .with_request_id_opt(request_id.map(str::to_string)),
        );
    }

    fn fire_trigger(
        &self,
        thread_id: u128,
        role: &str,
        content: &str,
    ) -> std::result::Result<(), String> {
        match &self.trigger {
            Some(trigger) => trigger.trigger(thread_id, role, content),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// In-memory fake: no I/O, no HTTP, no engine (FIRST).
    struct FakePorts {
        threads: Mutex<HashMap<u128, Vec<(String, String)>>>,
        created: Mutex<HashMap<u128, (String, Option<u64>)>>,
        next_id: Mutex<u128>,
        fail_create: Mutex<bool>,
        fail_send: Mutex<bool>,
        audits: Mutex<Vec<(u128, Option<String>)>>,
        triggers: Mutex<Vec<(u128, String, String)>>,
        trigger_err: Mutex<Option<String>>,
    }

    impl FakePorts {
        fn new() -> Self {
            Self {
                threads: Mutex::new(HashMap::new()),
                created: Mutex::new(HashMap::new()),
                next_id: Mutex::new(7),
                fail_create: Mutex::new(false),
                fail_send: Mutex::new(false),
                audits: Mutex::new(Vec::new()),
                triggers: Mutex::new(Vec::new()),
                trigger_err: Mutex::new(None),
            }
        }

        fn with_thread(id: u128) -> Self {
            let fake = Self::new();
            fake.threads.lock().unwrap().insert(id, Vec::new());
            fake
        }

        fn cmd(thread_id: Option<u128>) -> StartConversationCommand {
            StartConversationCommand {
                thread_id,
                title: None,
                ttl_secs: None,
                role: "user".to_string(),
                content: "hello".to_string(),
                request_id: Some("req-1".to_string()),
            }
        }
    }

    impl StartConversationPorts for FakePorts {
        fn create_thread(&self, title: &str, ttl_secs: Option<u64>) -> Result<u128> {
            if *self.fail_create.lock().unwrap() {
                return Err(Error::NodeNotFound(0));
            }
            let mut next = self.next_id.lock().unwrap();
            let id = *next;
            *next += 1;
            self.threads.lock().unwrap().insert(id, Vec::new());
            self.created
                .lock()
                .unwrap()
                .insert(id, (title.to_string(), ttl_secs));
            Ok(id)
        }

        fn send_message(&self, thread_id: u128, role: &str, content: &str) -> Result<()> {
            if *self.fail_send.lock().unwrap() {
                return Err(Error::NodeNotFound(thread_id));
            }
            match self.threads.lock().unwrap().get_mut(&thread_id) {
                Some(msgs) => {
                    msgs.push((role.to_string(), content.to_string()));
                    Ok(())
                }
                None => Err(Error::NodeNotFound(thread_id)),
            }
        }

        fn record_audit(&self, thread_id: u128, request_id: Option<&str>) {
            self.audits
                .lock()
                .unwrap()
                .push((thread_id, request_id.map(str::to_string)));
        }

        fn fire_trigger(
            &self,
            thread_id: u128,
            role: &str,
            content: &str,
        ) -> std::result::Result<(), String> {
            self.triggers
                .lock()
                .unwrap()
                .push((thread_id, role.to_string(), content.to_string()));
            match self.trigger_err.lock().unwrap().clone() {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }
    }

    #[test]
    fn creates_thread_with_default_title_when_id_absent() {
        // Arrange
        let ports = FakePorts::new();
        // Act
        let id = StartConversationUseCase::execute(&ports, FakePorts::cmd(None)).unwrap();
        // Assert
        assert_eq!(id, 7);
        assert_eq!(
            ports.created.lock().unwrap().get(&id),
            Some(&("conversation".to_string(), None))
        );
        assert_eq!(
            ports.threads.lock().unwrap().get(&id).unwrap(),
            &vec![("user".to_string(), "hello".to_string())]
        );
    }

    #[test]
    fn passes_custom_title_and_ttl_to_create() {
        // Arrange
        let ports = FakePorts::new();
        let mut cmd = FakePorts::cmd(None);
        cmd.title = Some("support".to_string());
        cmd.ttl_secs = Some(60);
        // Act
        let id = StartConversationUseCase::execute(&ports, cmd).unwrap();
        // Assert
        assert_eq!(
            ports.created.lock().unwrap().get(&id),
            Some(&("support".to_string(), Some(60)))
        );
    }

    #[test]
    fn reuses_existing_thread_without_creating() {
        // Arrange
        let ports = FakePorts::with_thread(42);
        let mut cmd = FakePorts::cmd(Some(42));
        cmd.title = Some("ignored".to_string());
        // Act
        let id = StartConversationUseCase::execute(&ports, cmd).unwrap();
        // Assert
        assert_eq!(id, 42);
        assert!(ports.created.lock().unwrap().is_empty());
        assert_eq!(ports.threads.lock().unwrap().get(&42).unwrap().len(), 1);
    }

    #[test]
    fn records_audit_with_request_id_and_fires_trigger() {
        // Arrange
        let ports = FakePorts::new();
        // Act
        let id = StartConversationUseCase::execute(&ports, FakePorts::cmd(None)).unwrap();
        // Assert
        assert_eq!(
            ports.audits.lock().unwrap().as_slice(),
            &[(id, Some("req-1".to_string()))]
        );
        assert_eq!(
            ports.triggers.lock().unwrap().as_slice(),
            &[(id, "user".to_string(), "hello".to_string())]
        );
    }

    #[test]
    fn records_audit_without_request_id_when_absent() {
        // Arrange
        let ports = FakePorts::new();
        let mut cmd = FakePorts::cmd(None);
        cmd.request_id = None;
        // Act
        let id = StartConversationUseCase::execute(&ports, cmd).unwrap();
        // Assert
        assert_eq!(ports.audits.lock().unwrap().as_slice(), &[(id, None)]);
    }

    #[test]
    fn propagates_create_error_without_side_effects() {
        // Arrange
        let ports = FakePorts::new();
        *ports.fail_create.lock().unwrap() = true;
        // Act
        let err = StartConversationUseCase::execute(&ports, FakePorts::cmd(None)).unwrap_err();
        // Assert
        assert!(matches!(err, Error::NodeNotFound(0)));
        assert!(ports.audits.lock().unwrap().is_empty());
        assert!(ports.triggers.lock().unwrap().is_empty());
    }

    #[test]
    fn propagates_send_error_without_audit_or_trigger() {
        // Arrange: unknown thread id, create is skipped so send fails.
        let ports = FakePorts::new();
        // Act
        let err = StartConversationUseCase::execute(&ports, FakePorts::cmd(Some(999))).unwrap_err();
        // Assert
        assert!(matches!(err, Error::NodeNotFound(999)));
        assert!(ports.audits.lock().unwrap().is_empty());
        assert!(ports.triggers.lock().unwrap().is_empty());
    }

    #[test]
    fn swallows_trigger_failure_and_still_returns_ok() {
        // Arrange
        let ports = FakePorts::new();
        *ports.trigger_err.lock().unwrap() = Some("pipeline down".to_string());
        // Act (P4: extraction failures never fail the request)
        let id = StartConversationUseCase::execute(&ports, FakePorts::cmd(None)).unwrap();
        // Assert
        assert_eq!(id, 7);
        assert_eq!(ports.triggers.lock().unwrap().len(), 1);
        assert_eq!(ports.audits.lock().unwrap().len(), 1);
    }
}
