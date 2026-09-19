use serde::Serialize;

/// 업로드 응답. 본문 마크다운에는 이 `url` 을 그대로 삽입한다.
/// 상대 경로(`/api/images/{storedName}`)로 준다 — 도메인이 나중에 바뀌어도 이미 저장된
/// 게시물의 이미지가 깨지지 않도록. Discord 알림처럼 외부에서 접근 가능한 절대 URL이 필요한
/// 곳은 알림을 보내는 시점에 그때그때 변환한다 (`services::discord::resolve_image_url`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadedImageDto {
    pub url: String,
}
