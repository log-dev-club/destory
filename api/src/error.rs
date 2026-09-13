use axum::{
    Json,
    extract::multipart::MultipartError,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tracing::error;

/// 모든 핸들러가 반환하는 공통 에러. `{ "error": "<메시지>" }` + HTTP 상태 코드로 변환된다.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("로그인이 필요합니다")]
    Unauthorized,
    #[error("권한이 없습니다")]
    Forbidden,
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    PayloadTooLarge(String),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        // 내부 오류는 상세 내용을 로그에만 남기고 클라이언트에는 일반 메시지를 준다
        let message = match &self {
            Self::Internal(detail) => {
                error!(detail, "internal error");
                "서버 내부 오류가 발생했습니다".to_string()
            },
            other => other.to_string(),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        if let Some(db) = e.as_database_error() {
            // 23505: unique_violation, 23503: foreign_key_violation
            match db.code().as_deref() {
                Some("23505") => return Self::Conflict("이미 존재하는 값입니다".into()),
                Some("23503") => return Self::NotFound("참조하는 대상이 없습니다".into()),
                _ => {},
            }
        }
        Self::Internal(format!("database error: {e}"))
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::Internal(format!("io error: {e}"))
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(e: tokio::task::JoinError) -> Self {
        Self::Internal(format!("task join error: {e}"))
    }
}

impl From<MultipartError> for AppError {
    fn from(e: MultipartError) -> Self {
        if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
            Self::PayloadTooLarge("첨부파일이 허용 크기를 초과했습니다".into())
        } else {
            Self::BadRequest(format!("multipart 파싱 실패: {}", e.body_text()))
        }
    }
}
