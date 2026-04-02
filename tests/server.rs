use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use iadbms::server::{app, ServerState, QueryRequest};
use iadbms::storage::StorageEngine;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check() {
    let storage = Arc::new(StorageEngine::new());
    let state = Arc::new(ServerState { storage });
    let app = app(state);

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
