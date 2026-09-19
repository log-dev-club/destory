//! 게시물 본문 삽입용 이미지 업로드(multipart)/서빙

use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

use crate::{
    error::AppError,
    extract::CurrentUser,
    models::image::UploadedImageDto,
    services::{self, images::file_path},
    state::AppState,
};

/// POST /api/images (multipart/form-data, 필드: `file`). 로그인한 사용자만.
pub async fn upload(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadedImageDto>), AppError> {
    let images_dir = state.config.upload_dir.join("images");
    tokio::fs::create_dir_all(&images_dir).await?;

    let mut stored_name = None;
    let mut size: i64 = 0;
    let mut tmp_path: Option<std::path::PathBuf> = None;

    while let Some(mut field) = multipart.next_field().await? {
        if field.name() != Some("file") {
            continue;
        }
        if stored_name.is_some() {
            return Err(AppError::bad_request(
                "파일은 요청당 하나만 첨부할 수 있습니다",
            ));
        }
        let name = field
            .file_name()
            .ok_or_else(|| AppError::bad_request("파일 이름이 없습니다"))?
            .to_string();
        services::images::content_type_for(&name)?;
        let generated = services::images::random_stored_name(&name);
        let path = file_path(&state.config.upload_dir, &generated);

        let mut file = tokio::fs::File::create(&path).await?;
        while let Some(chunk) = field.chunk().await? {
            file.write_all(&chunk).await?;
            size += chunk.len() as i64;
        }
        file.flush().await?;

        tmp_path = Some(path);
        stored_name = Some(generated);
    }

    let stored_name = stored_name.ok_or_else(|| AppError::bad_request("file 필드가 없습니다"))?;

    let created = services::images::create(&state.pool, user.id, &stored_name, size).await;
    if created.is_err()
        && let Some(path) = tmp_path
    {
        let _ = tokio::fs::remove_file(path).await;
    }
    created?;

    // 상대 경로로 저장해야 도메인이 바뀌어도(예: IP → 커스텀 도메인) 예전 게시물의 이미지가 깨지지 않는다.
    // 외부에 절대 URL이 필요한 경우(Discord 썸네일)는 그때그때 현재 APP_BASE_URL 로 변환한다
    // (services::discord::resolve_image_url 참고).
    let url = format!("/api/images/{stored_name}");
    Ok((StatusCode::CREATED, Json(UploadedImageDto { url })))
}

/// GET /api/images/{stored_name} — 공개, 인증 불필요
pub async fn serve(
    State(state): State<AppState>,
    Path(stored_name): Path<String>,
) -> Result<Response, AppError> {
    if !services::images::exists(&state.pool, &stored_name).await? {
        return Err(AppError::not_found("이미지를 찾을 수 없습니다"));
    }
    let content_type = services::images::content_type_for(&stored_name)?;
    let path = file_path(&state.config.upload_dir, &stored_name);

    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|_| AppError::not_found("파일이 서버에 존재하지 않습니다"))?;
    let len = file.metadata().await?.len();

    let mut response = Body::from_stream(ReaderStream::new(file)).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    headers.insert(header::CONTENT_LENGTH, HeaderValue::from(len));
    // 브라우저/Discord 가 매번 새로 받지 않도록 캐시 허용 (내용이 바뀌지 않는 불변 파일)
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    Ok(response)
}
