use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{
    BatchCreateBusinessHoursRequest, CreateBusinessHoursRequest,
};
use crate::store::{
    add_business_hours, batch_add_business_hours, create_store, get_business_hours,
    get_business_status, get_store, AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStoreRequest {
    pub name: String,
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    pub fn success_no_data() -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
            data: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            code: 1,
            message: message.to_string(),
            data: None,
        }
    }
}

pub async fn health_check() -> impl IntoResponse {
    let response = ApiResponse::<String> {
        code: 0,
        message: "Service is running".to_string(),
        data: None,
    };
    (StatusCode::OK, Json(response))
}

pub async fn create_store_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateStoreRequest>,
) -> impl IntoResponse {
    if req.name.trim().is_empty() {
        let response: ApiResponse<String> = ApiResponse::error("门店名称不能为空");
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    let store = create_store(&state, req.name, req.address).await;
    (StatusCode::CREATED, Json(ApiResponse::success(store)))
}

pub async fn get_store_handler(
    State(state): State<AppState>,
    Path(store_id): Path<Uuid>,
) -> impl IntoResponse {
    match get_store(&state, store_id).await {
        Some(store) => (StatusCode::OK, Json(ApiResponse::success(store))),
        None => {
            let response: ApiResponse<String> = ApiResponse::error("门店不存在");
            (StatusCode::NOT_FOUND, Json(response))
        }
    }
}

pub async fn add_business_hours_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateBusinessHoursRequest>,
) -> impl IntoResponse {
    if !req.is_closed && req.close_time == req.open_time {
        let response: ApiResponse<String> = ApiResponse::error("结束时间不能等于开始时间");
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    match add_business_hours(&state, req).await {
        Ok(hours) => (StatusCode::CREATED, Json(ApiResponse::success(hours))),
        Err(e) => {
            let response: ApiResponse<String> = ApiResponse::error(&e);
            (StatusCode::NOT_FOUND, Json(response))
        }
    }
}

pub async fn batch_add_business_hours_handler(
    State(state): State<AppState>,
    Json(req): Json<BatchCreateBusinessHoursRequest>,
) -> impl IntoResponse {
    if req.hours.is_empty() {
        let response: ApiResponse<String> = ApiResponse::error("营业时间列表不能为空");
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    for item in &req.hours {
        if !item.is_closed && item.close_time == item.open_time {
            let response: ApiResponse<String> = ApiResponse::error("营业时间配置错误：结束时间不能等于开始时间");
            return (StatusCode::BAD_REQUEST, Json(response));
        }
    }

    match batch_add_business_hours(&state, req).await {
        Ok(hours) => (StatusCode::CREATED, Json(ApiResponse::success(hours))),
        Err(e) => {
            let response: ApiResponse<String> = ApiResponse::error(&e);
            (StatusCode::NOT_FOUND, Json(response))
        }
    }
}

pub async fn get_business_hours_handler(
    State(state): State<AppState>,
    Path(store_id): Path<Uuid>,
) -> impl IntoResponse {
    match get_business_hours(&state, store_id).await {
        Ok(hours) => (StatusCode::OK, Json(ApiResponse::success(hours))),
        Err(e) => {
            let response: ApiResponse<String> = ApiResponse::error(&e);
            (StatusCode::NOT_FOUND, Json(response))
        }
    }
}

pub async fn get_business_status_handler(
    State(state): State<AppState>,
    Path(store_id): Path<Uuid>,
) -> impl IntoResponse {
    match get_business_status(&state, store_id).await {
        Ok(status) => (StatusCode::OK, Json(ApiResponse::success(status))),
        Err(e) => {
            let response: ApiResponse<String> = ApiResponse::error(&e);
            (StatusCode::NOT_FOUND, Json(response))
        }
    }
}
