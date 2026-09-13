use serde::Deserialize;

/// POST /api/auth/register
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub nickname: String,
    pub password: String,
}

/// POST /api/auth/login
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub nickname: String,
    pub password: String,
}
