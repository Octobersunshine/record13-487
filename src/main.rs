use std::net::SocketAddr;

use axum::{
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::{
    add_business_hours_handler, batch_add_business_hours_handler, create_store_handler,
    get_business_hours_handler, get_business_status_handler, get_store_handler, health_check,
    ApiResponse,
};
use crate::store::AppState;

mod handlers;
mod models;
mod store;

async fn not_found() -> impl IntoResponse {
    let response: ApiResponse<String> = ApiResponse {
        code: 404,
        message: "Not Found".to_string(),
        data: None,
    };
    (StatusCode::NOT_FOUND, Json(response))
}

fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_origin(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_check))
        .route("/stores", post(create_store_handler))
        .route("/stores/:store_id", get(get_store_handler))
        .route("/stores/:store_id/hours", post(add_business_hours_handler))
        .route(
            "/stores/:store_id/hours/batch",
            post(batch_add_business_hours_handler),
        )
        .route("/stores/:store_id/hours", get(get_business_hours_handler))
        .route(
            "/stores/:store_id/status",
            get(get_business_status_handler),
        )
        .with_state(state)
        .fallback(not_found)
        .layer(cors)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let state = AppState::new();
    state.seed_demo_data().await;

    let stores = state.inner.stores.read().await;
    if let Some((id, _)) = stores.iter().next() {
        tracing::info!("示例门店 ID: {}", id);
    }
    drop(stores);

    let app = create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("服务器启动，监听地址: {}", addr);
    tracing::info!("API 文档:");
    tracing::info!("  GET  /health                           - 健康检查");
    tracing::info!("  POST /stores                           - 创建门店");
    tracing::info!("  GET  /stores/:store_id                 - 查询门店信息");
    tracing::info!("  POST /stores/:store_id/hours           - 新增单个营业时间段");
    tracing::info!("  POST /stores/:store_id/hours/batch     - 批量新增营业时间段");
    tracing::info!("  GET  /stores/:store_id/hours           - 查询门店营业时间段");
    tracing::info!("  GET  /stores/:store_id/status          - 查询门店营业状态");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
