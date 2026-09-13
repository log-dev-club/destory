-- 초기 스키마: 사용자, 게시물, 태그, 댓글

CREATE TABLE users (
    id          BIGSERIAL PRIMARY KEY,
    user_name   VARCHAR(50)  NOT NULL UNIQUE,   -- 헤더 필터(user_name)에 사용
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE TABLE posts (
    id           BIGSERIAL PRIMARY KEY,
    user_id      BIGINT       NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title        VARCHAR(200) NOT NULL,
    summary      VARCHAR(300) NOT NULL,         -- Discord 알림용 간략 설명
    content      TEXT         NOT NULL,         -- GFM Markdown 원문
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- 메인 화면 최신순 조회용
CREATE INDEX idx_posts_created_at ON posts (created_at DESC);

CREATE TABLE tags (
    id    BIGSERIAL   PRIMARY KEY,
    name  VARCHAR(50) NOT NULL UNIQUE
);

CREATE TABLE post_tags (
    post_id  BIGINT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    tag_id   BIGINT NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (post_id, tag_id)
);

CREATE TABLE comments (
    id          BIGSERIAL PRIMARY KEY,
    post_id     BIGINT      NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id     BIGINT      NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content     TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_comments_post_id ON comments (post_id);
