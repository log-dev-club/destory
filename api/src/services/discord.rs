//! 게시물 업로드 시 Discord Webhook 알림.
//! 실패해도 게시물 저장에는 영향을 주지 않도록 별도 태스크에서 보내고 로그만 남긴다.

use serde_json::json;
use tracing::{debug, info, warn};

use crate::{models::post::PostDetail, state::AppState};

const EMBED_COLOR: u32 = 0x00F0_A04B; // 프론트엔드 accent 색상 (#F0A04B)

pub fn notify_new_post(state: &AppState, post: &PostDetail) {
    let Some(url) = state.config.discord_webhook_url.clone() else {
        debug!("DISCORD_WEBHOOK_URL 미설정: 알림 건너뜀");
        return;
    };

    let link = format!("{}/posts/{}", state.config.app_base_url, post.summary.id);
    let tags = post
        .summary
        .tags
        .iter()
        .map(|t| format!("#{t}"))
        .collect::<Vec<_>>()
        .join(" ");

    let payload = json!({
        "embeds": [{
            "title": post.summary.title,
            "description": post.summary.excerpt,
            "url": link,
            "color": EMBED_COLOR,
            "author": { "name": post.summary.author.nickname },
            "footer": { "text": if tags.is_empty() { "새 게시물".to_string() } else { tags } },
        }]
    });

    let client = state.http.clone();
    let post_id = post.summary.id;
    tokio::spawn(async move {
        match client.post(&url).json(&payload).send().await {
            Ok(res) if res.status().is_success() => {
                info!(post_id, "Discord 알림 전송 완료");
            },
            Ok(res) => {
                warn!(post_id, status = %res.status(), "Discord 알림 실패 (응답 오류)");
            },
            Err(e) => {
                warn!(post_id, error = %e, "Discord 알림 실패 (요청 오류)");
            },
        }
    });
}
