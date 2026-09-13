//! 게시물 업로드 시 Discord Webhook 알림.
//! 실패해도 게시물 저장에는 영향을 주지 않도록 별도 태스크에서 보내고 로그만 남긴다.

use serde_json::{Value, json};
use tracing::{debug, info, warn};

use crate::{models::post::PostDetail, state::AppState};

const EMBED_COLOR: u32 = 0x00F0_A04B; // 프론트엔드 accent 색상 (#F0A04B)

/// Discord 가 외부에서 가져올 수 있는 공개 URL 인지 (http/https 만, 로컬 주소 제외)
fn is_public_image_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    (lower.starts_with("https://") || lower.starts_with("http://"))
        && !lower.contains("localhost")
        && !lower.contains("127.0.0.1")
}

/// 마크다운 본문에서 첫 번째 이미지 `![alt](url "title")` 의 url 을 찾는다.
/// 코드 블록(```) 안은 건너뛴다. 공개 URL 이 아니면 None.
pub fn first_image_url(markdown: &str) -> Option<String> {
    let mut in_code = false;
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }

        let mut rest = line;
        while let Some(start) = rest.find("![") {
            let after_bang = &rest[start + 2..];
            let Some(close_bracket) = after_bang.find("](") else {
                break;
            };
            let after_paren = &after_bang[close_bracket + 2..];
            let Some(close_paren) = after_paren.find(')') else {
                break;
            };
            // `url "title"` 형태면 공백 앞까지만 url
            let url = after_paren[..close_paren]
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches(|c| c == '<' || c == '>');
            if is_public_image_url(url) {
                return Some(url.to_string());
            }
            rest = &after_paren[close_paren + 1..];
        }
    }
    None
}

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

    let mut embed = json!({
        "title": post.summary.title,
        "description": post.summary.excerpt,
        "url": link,
        "color": EMBED_COLOR,
        "author": { "name": post.summary.author.nickname },
        "footer": { "text": if tags.is_empty() { "새 게시물".to_string() } else { tags } },
    });

    // 본문의 첫 이미지를 카드 오른쪽 썸네일로
    if let Some(image) = first_image_url(&post.content) {
        embed["thumbnail"] = json!({ "url": image });
    }
    // 작성자 아바타가 공개 URL 이면 작성자 이름 옆 아이콘으로
    if let Some(avatar) = post
        .summary
        .author
        .avatar_url
        .as_deref()
        .filter(|u| is_public_image_url(u))
    {
        embed["author"]["icon_url"] = Value::String(avatar.to_string());
    }

    let payload = json!({ "embeds": [embed] });

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

#[cfg(test)]
mod tests {
    use super::first_image_url;

    #[test]
    fn finds_first_markdown_image() {
        let md = "# 제목\n\n설명 ![스크린샷](https://example.com/a.png) 뒤 ![b](https://example.com/b.png)";
        assert_eq!(
            first_image_url(md).as_deref(),
            Some("https://example.com/a.png")
        );
    }

    #[test]
    fn strips_title_and_angle_brackets() {
        let md = "![a](<https://example.com/a.png> \"제목\")";
        assert_eq!(
            first_image_url(md).as_deref(),
            Some("https://example.com/a.png")
        );
    }

    #[test]
    fn skips_code_blocks_and_local_urls() {
        let md = "```md\n![코드](https://example.com/in-code.png)\n```\n![로컬](http://localhost:5173/x.png)\n![상대](/img/y.png)\n![공개](https://cdn.example.com/z.jpg)";
        assert_eq!(
            first_image_url(md).as_deref(),
            Some("https://cdn.example.com/z.jpg")
        );
    }

    #[test]
    fn none_when_no_image() {
        assert_eq!(
            first_image_url("이미지 없음 [링크](https://example.com)"),
            None
        );
    }
}
