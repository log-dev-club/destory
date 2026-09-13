//! 첨부파일 업로드(multipart)/다운로드(stream)/삭제

use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use rand::RngCore;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

use crate::{
    error::AppError,
    extract::CurrentUser,
    models::attachment::AttachmentDto,
    services::{self, attachments::file_path, to_hex},
    state::AppState,
};

/// multipart 로 받은 필드 모음
#[derive(Default)]
struct UploadFields {
    original_name: Option<String>,
    hashed_name: Option<String>,
    sent_at: Option<i64>,
    size: i64,
    file_saved: bool,
}

/// multipart 본문을 읽어 파일은 임시 경로에 스트리밍 저장하고 나머지 필드를 모은다
async fn read_multipart(
    multipart: &mut Multipart,
    tmp_path: &std::path::Path,
) -> Result<UploadFields, AppError> {
    let mut fields = UploadFields::default();

    while let Some(mut field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "hashedName" => fields.hashed_name = Some(field.text().await?.trim().to_string()),
            "sentAt" => {
                let text = field.text().await?;
                fields.sent_at =
                    Some(text.trim().parse().map_err(|_| {
                        AppError::bad_request("sentAt 은 밀리초 단위 정수여야 합니다")
                    })?);
            },
            "file" => {
                if fields.file_saved {
                    return Err(AppError::bad_request(
                        "파일은 요청당 하나만 첨부할 수 있습니다",
                    ));
                }
                fields.original_name = field.file_name().map(str::to_string);
                let mut file = tokio::fs::File::create(tmp_path).await?;
                while let Some(chunk) = field.chunk().await? {
                    file.write_all(&chunk).await?;
                    fields.size += chunk.len() as i64;
                }
                file.flush().await?;
                fields.file_saved = true;
            },
            _ => {},
        }
    }
    Ok(fields)
}

/// POST /api/posts/{id}/attachments (multipart/form-data)
/// 필드: `file` (필수), `hashedName` (필수), `sentAt` (필수, ms epoch). 게시물 작성자만.
pub async fn upload(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(post_id): Path<i64>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<AttachmentDto>), AppError> {
    services::posts::ensure_owner(&state.pool, post_id, user.id).await?;

    let upload_dir = &state.config.upload_dir;
    let mut random = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut random);
    let tmp_path = upload_dir.join(format!("tmp-{}", to_hex(&random)));

    let result = async {
        let fields = read_multipart(&mut multipart, &tmp_path).await?;

        if !fields.file_saved {
            return Err(AppError::bad_request("file 필드가 없습니다"));
        }
        let original_name = fields
            .original_name
            .ok_or_else(|| AppError::bad_request("파일 이름이 없습니다"))?;
        let hashed_name = fields
            .hashed_name
            .ok_or_else(|| AppError::bad_request("hashedName 필드가 없습니다"))?;
        let sent_at_ms = fields
            .sent_at
            .ok_or_else(|| AppError::bad_request("sentAt 필드가 없습니다"))?;

        let sent_at =
            services::attachments::validate_upload(&original_name, &hashed_name, sent_at_ms)?;

        let final_path = file_path(upload_dir, &hashed_name);
        if tokio::fs::try_exists(&final_path).await? {
            return Err(AppError::conflict(
                "같은 hashedName 의 파일이 이미 있습니다",
            ));
        }
        tokio::fs::rename(&tmp_path, &final_path).await?;

        let created = services::attachments::create(
            &state.pool,
            post_id,
            &original_name,
            &hashed_name,
            fields.size,
            sent_at,
        )
        .await;

        if created.is_err() {
            services::attachments::remove_file(upload_dir, &hashed_name).await;
        }
        created
    }
    .await;

    // 어떤 경로로 실패했든 임시 파일은 남기지 않는다
    let _ = tokio::fs::remove_file(&tmp_path).await;

    result.map(|dto| (StatusCode::CREATED, Json(dto)))
}

/// GET /api/posts/{id}/attachments
pub async fn list(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
) -> Result<Json<Vec<AttachmentDto>>, AppError> {
    services::posts::owner_id(&state.pool, post_id).await?; // 존재 확인
    Ok(Json(
        services::attachments::list_for_post(&state.pool, post_id).await?,
    ))
}

/// RFC 5987 filename* 용 percent-encoding (비예약 문자 외 전부 인코딩)
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// 구형 클라이언트용 ASCII 대체 파일명
fn ascii_fallback(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// GET /api/attachments/{hashedName} — 원본 파일명으로 다운로드
pub async fn download(
    State(state): State<AppState>,
    Path(hashed_name): Path<String>,
) -> Result<Response, AppError> {
    // DB 에 등록된 hashed_name 만 디스크 경로로 사용한다 (경로 조작 방지)
    let row = services::attachments::by_hashed_name(&state.pool, &hashed_name).await?;
    let path = file_path(&state.config.upload_dir, &row.hashed_name);

    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|_| AppError::not_found("파일이 서버에 존재하지 않습니다"))?;
    let len = file.metadata().await?.len();

    let disposition = format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        ascii_fallback(&row.original_name),
        percent_encode(&row.original_name)
    );

    let mut response = Body::from_stream(ReaderStream::new(file)).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition)
            .map_err(|e| AppError::Internal(format!("invalid header: {e}")))?,
    );
    headers.insert(header::CONTENT_LENGTH, HeaderValue::from(len));
    Ok(response)
}

/// DELETE /api/attachments/{hashedName} — 게시물 작성자만
pub async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(hashed_name): Path<String>,
) -> Result<StatusCode, AppError> {
    services::attachments::delete(&state.pool, &hashed_name, user.id, &state.config.upload_dir)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
