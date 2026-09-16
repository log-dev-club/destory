use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

use super::{attachment::AttachmentDto, user::Author};

/// 목록/상세 공용 SELECT 결과 (services::posts 의 기본 쿼리와 컬럼명이 일치해야 함)
#[derive(Debug, FromRow)]
pub struct PostRow {
    pub id: i64,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub tags: Vec<String>,
    pub comment_count: i64,
    pub star_count: i64,
    /// 요청한 사용자가 별을 눌렀는지 (비로그인은 false)
    pub starred: bool,
    /// 필터 적용 후 전체 건수 (count(*) OVER ())
    pub total: i64,
}

/// 목록 항목. 프론트엔드 `Post` 타입에서 `content` 만 뺀 형태.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostSummary {
    pub id: i64,
    pub title: String,
    pub excerpt: String,
    pub author: Author,
    pub tags: Vec<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub comment_count: i64,
    pub star_count: i64,
    pub starred: bool,
    /// 본문에 마크다운 이미지(`![...](...)`)가 하나라도 있는지. 홈 목록의 이미지 아이콘 표시용
    pub has_image: bool,
}

/// 상세 응답. 프론트엔드 `Post` 타입 + attachments
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostDetail {
    #[serde(flatten)]
    pub summary: PostSummary,
    pub content: String,
    pub attachments: Vec<AttachmentDto>,
}

impl PostRow {
    pub fn into_summary(self) -> PostSummary {
        let has_image = self.content.contains("![");
        PostSummary {
            id: self.id,
            title: self.title,
            excerpt: self.summary,
            author: Author {
                nickname: self.nickname,
                avatar_url: self.avatar_url,
            },
            tags: self.tags,
            created_at: self.created_at,
            updated_at: self.updated_at,
            comment_count: self.comment_count,
            star_count: self.star_count,
            starred: self.starred,
            has_image,
        }
    }

    pub fn into_detail(self, attachments: Vec<AttachmentDto>) -> PostDetail {
        let content = self.content.clone();
        PostDetail {
            summary: self.into_summary(),
            content,
            attachments,
        }
    }
}

/// GET /api/posts 응답
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostPage {
    pub items: Vec<PostSummary>,
    pub page: u32,
    pub limit: u32,
    pub total: i64,
}

/// GET /api/posts 쿼리 파라미터.
/// 프론트엔드 `parseSearchQuery` 의 tag:/user:/자유 텍스트를 각각 `tag`, `user`, `q` 로 넘긴다.
#[derive(Debug, Default, Deserialize)]
pub struct PostListQuery {
    /// 쉼표로 구분한 태그. 모두 포함(부분 일치)해야 함. 예: `react,frontend`
    pub tag: Option<String>,
    /// 작성자 닉네임 부분 일치
    pub user: Option<String>,
    /// 공백으로 구분한 검색어. 제목·요약·닉네임·태그 중 하나에 모두 포함되어야 함
    pub q: Option<String>,
    /// 1부터 시작 (기본 1)
    pub page: Option<u32>,
    /// 페이지 크기 (기본 20, 최대 100)
    pub limit: Option<u32>,
}

/// 서비스 계층에 넘기는 정규화된 필터
#[derive(Debug, Clone)]
pub struct PostFilter {
    pub tags: Vec<String>,
    pub user: Option<String>,
    pub text: Vec<String>,
    /// 특정 작성자의 글만 (마이페이지)
    pub author_id: Option<i64>,
    pub page: u32,
    pub limit: u32,
}

fn split_tokens(value: Option<String>, sep: fn(char) -> bool) -> Vec<String> {
    value
        .map(|v| {
            v.split(sep)
                .map(|t| t.trim().to_lowercase())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

impl PostListQuery {
    pub fn into_filter(self, author_id: Option<i64>) -> PostFilter {
        PostFilter {
            tags: split_tokens(self.tag, |c| c == ','),
            user: self
                .user
                .map(|u| u.trim().to_string())
                .filter(|u| !u.is_empty()),
            text: split_tokens(self.q, char::is_whitespace),
            author_id,
            page: self.page.unwrap_or(1).max(1),
            limit: self.limit.unwrap_or(20).clamp(1, 100),
        }
    }
}

/// POST /api/posts
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePostRequest {
    pub title: String,
    /// GFM Markdown 원문
    pub content: String,
    /// 목록·Discord 알림용 요약. 생략하면 본문 앞부분에서 자동 생성
    pub excerpt: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// PATCH /api/posts/{id}. 생략한 필드는 유지, `tags` 를 보내면 전체 교체
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// PUT/DELETE /api/posts/{id}/star 응답
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StarResponse {
    pub starred: bool,
    pub star_count: i64,
}
