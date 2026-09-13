//! DB row 구조체(sqlx::FromRow)와 요청/응답 DTO(serde).
//! JSON 필드는 프론트엔드 `types.ts` 와 맞추기 위해 camelCase 로 직렬화한다.

pub mod attachment;
pub mod auth;
pub mod comment;
pub mod draft;
pub mod post;
pub mod tag;
pub mod user;
