-- 게시글 임시저장 (사용자당 1개, 새로 저장하면 덮어씀)

CREATE TABLE post_drafts (
    user_id     BIGINT      PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    title       TEXT        NOT NULL DEFAULT '',
    content     TEXT        NOT NULL DEFAULT '',
    tags_input  TEXT        NOT NULL DEFAULT '',  -- 태그 입력창 원문 그대로 (쉼표 구분, 파싱은 프론트에서)
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
