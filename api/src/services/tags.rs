use sqlx::PgPool;

use crate::{error::AppError, models::tag::TagCount};

/// 사용 중인 태그와 게시물 수 (많이 쓰인 순). 게시물이 하나도 없는 태그는 제외.
pub async fn list(pool: &PgPool) -> Result<Vec<TagCount>, AppError> {
    let rows = sqlx::query_as::<_, TagCount>(
        "SELECT t.name::text AS name, count(pt.post_id) AS post_count \
         FROM tags t JOIN post_tags pt ON pt.tag_id = t.id \
         GROUP BY t.id ORDER BY post_count DESC, t.name ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
