use serde::Serialize;

/// 업로드 응답. 본문 마크다운에는 이 `url` 을 그대로 삽입한다 (Discord 등 외부에서 접근 가능한 절대 URL).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadedImageDto {
    pub url: String,
}
