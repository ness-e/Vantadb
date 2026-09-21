//! SkillOversizeStrategy (MEM-17) — last-resort split for oversized buffers.
//!
//! Port of TDAM `conversation-add/oversize-strategy.ts`: when the compressed
//! buffer still exceeds `chunk_max_bytes`, keep messages from the head up to
//! `head_keep_bytes` and from the tail up to `tail_keep_bytes` (always at
//! least one message each side), replacing the omitted middle with a single
//! `system` placeholder message.

use super::compressor::SkillMessage;

/// Oversize tuning (TDAM defaults: 80KB chunk, 20KB head/tail).
#[derive(Debug, Clone)]
pub struct OversizeOptions {
    pub chunk_max_bytes: usize,
    pub head_keep_bytes: usize,
    pub tail_keep_bytes: usize,
}

pub const OVERSIZE_DEFAULTS: OversizeOptions = OversizeOptions {
    chunk_max_bytes: 81_920,
    head_keep_bytes: 20_480,
    tail_keep_bytes: 20_480,
};

/// Outcome of one oversize pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OversizeResult {
    pub messages: Vec<SkillMessage>,
    /// `false` = passthrough (nothing omitted).
    pub truncated: bool,
    pub omitted_message_count: usize,
    pub omitted_bytes: usize,
}

fn message_bytes(msg: &SkillMessage) -> usize {
    serde_json::to_string(msg).map_or(0, |s| s.len())
}

fn total_bytes(msgs: &[SkillMessage]) -> usize {
    msgs.iter().map(message_bytes).sum()
}

/// Apply the oversize strategy (TDAM `applyOversizeStrategy`).
pub fn apply_oversize_strategy(
    messages: &[SkillMessage],
    opts: &OversizeOptions,
) -> OversizeResult {
    if messages.is_empty() || total_bytes(messages) <= opts.chunk_max_bytes {
        return OversizeResult {
            messages: messages.to_vec(),
            truncated: false,
            omitted_message_count: 0,
            omitted_bytes: 0,
        };
    }

    // Accumulate from the head (at least one message, even if it alone
    // exceeds the budget).
    let mut head_end = 0usize;
    let mut head_bytes = 0usize;
    for (i, m) in messages.iter().enumerate() {
        let b = message_bytes(m);
        if head_end > 0 && head_bytes + b > opts.head_keep_bytes {
            break;
        }
        head_end = i + 1;
        head_bytes += b;
        if head_bytes >= opts.head_keep_bytes {
            break;
        }
    }

    // Accumulate from the tail without eating into the head region.
    let mut tail_start = messages.len();
    let mut tail_bytes = 0usize;
    for i in (head_end..messages.len()).rev() {
        let b = message_bytes(&messages[i]);
        if tail_start < messages.len() && tail_bytes + b > opts.tail_keep_bytes {
            break;
        }
        tail_start = i;
        tail_bytes += b;
        if tail_bytes >= opts.tail_keep_bytes {
            break;
        }
    }

    let omitted_count = tail_start.saturating_sub(head_end);
    if omitted_count == 0 {
        // Head+tail cover everything → passthrough.
        return OversizeResult {
            messages: messages.to_vec(),
            truncated: false,
            omitted_message_count: 0,
            omitted_bytes: 0,
        };
    }
    let omitted_bytes = total_bytes(&messages[head_end..tail_start]);

    let mut out = Vec::with_capacity(head_end + 1 + (messages.len() - tail_start));
    out.extend_from_slice(&messages[..head_end]);
    out.push(SkillMessage {
        role: "system".into(),
        content: format!(
            "[{omitted_count} middle messages / {omitted_bytes} bytes omitted — too long]"
        ),
    });
    out.extend_from_slice(&messages[tail_start..]);

    OversizeResult {
        messages: out,
        truncated: true,
        omitted_message_count: omitted_count,
        omitted_bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(content: &str) -> SkillMessage {
        SkillMessage::new("user", content)
    }

    #[test]
    fn small_buffer_passthrough() {
        let msgs = vec![msg("tiny")];
        let r = apply_oversize_strategy(&msgs, &OVERSIZE_DEFAULTS);
        assert!(!r.truncated);
        assert_eq!(r.messages.len(), 1);
    }

    #[test]
    fn big_buffer_keeps_head_tail_and_placeholder() {
        let unit = "x".repeat(1_000); // ~1KB serialized per message
        let msgs: Vec<SkillMessage> = (0..100).map(|_| msg(&unit)).collect();
        let opts = OversizeOptions {
            chunk_max_bytes: 10_000,
            head_keep_bytes: 2_500,
            tail_keep_bytes: 2_500,
        };
        let r = apply_oversize_strategy(&msgs, &opts);
        assert!(r.truncated);
        assert_eq!(r.messages.first().unwrap().content, unit);
        assert_eq!(r.messages.last().unwrap().content, unit);
        // ~1028 bytes serialized per message: head keeps 2, tail keeps 2,
        // middle replaced by one placeholder.
        assert_eq!(r.messages.len(), 5);
        assert_eq!(r.omitted_message_count, 96);
        assert!(r.messages[2].content.contains("middle messages"));
    }

    #[test]
    fn head_and_tail_always_keep_at_least_one_message_each() {
        // ponytail ceiling (TDAM parity): when head+tail cover EVERY message
        // (e.g. two giant messages), nothing is omitted → passthrough. The
        // strategy never fabricates an empty middle.
        let giant = msg(&"y".repeat(50_000));
        let opts = OversizeOptions {
            chunk_max_bytes: 10_000,
            head_keep_bytes: 1_000,
            tail_keep_bytes: 1_000,
        };
        let r = apply_oversize_strategy(&[giant.clone(), giant.clone()], &opts);
        assert!(!r.truncated, "head+tail cover all → passthrough");

        // With a middle section, the giants anchor head and tail and the
        // middle is omitted.
        let r = apply_oversize_strategy(
            &[
                giant.clone(),
                msg("m1"),
                msg("m2"),
                msg("m3"),
                giant.clone(),
            ],
            &opts,
        );
        assert!(r.truncated);
        assert_eq!(r.omitted_message_count, 3);
        assert_eq!(r.messages.len(), 3);
    }
}
