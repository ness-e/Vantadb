//! API Server & Health Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{TerminalReporter, VantaHarness};
use std::sync::Arc;
use tower::ServiceExt;
use vantadb::server::{app, ServerState};
use vantadb::storage::StorageEngine;

#[tokio::test]
async fn api_server_certification() {
    let mut harness = VantaHarness::new("API LAYER (SERVER & HEALTH)");

    harness.execute("Health: Endpoint Availability & Router State", || {
        futures::executor::block_on(async {
            let temp_dir = tempfile::tempdir().unwrap();
            let storage = Arc::new(StorageEngine::open(temp_dir.path().to_str().unwrap()).unwrap());
            let state = Arc::new(ServerState { storage });
            let app = app(state);

            TerminalReporter::sub_step("Dispatching oneshot request to /health...");
            let response = app
                .oneshot(
                    Request::builder()
                        .uri("/health")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
            TerminalReporter::success("API Health check passed.");
        });
    });
}
