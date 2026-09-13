-- 인증(세션), 프로필 필드, 별(star), 수정 시각 추가
-- 프론트엔드 types.ts 의 Author.nickname / Post.starCount 와 맞추기 위한 변경

-- users.user_name → nickname (프론트엔드 필드명과 통일)
ALTER TABLE users RENAME COLUMN user_name TO nickname;

ALTER TABLE users
    ADD COLUMN password_hash TEXT         NOT NULL DEFAULT '',  -- argon2 해시. 기존 행 호환용 기본값
    ADD COLUMN avatar_url    VARCHAR(500),
    ADD COLUMN bio           VARCHAR(300),
    ADD COLUMN updated_at    TIMESTAMPTZ  NOT NULL DEFAULT now();

-- 로그인 세션 (HttpOnly 쿠키에 token 저장)
CREATE TABLE sessions (
    token       CHAR(64)    PRIMARY KEY,           -- 32바이트 난수의 hex 문자열
    user_id     BIGINT      NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sessions_user_id ON sessions (user_id);

-- 게시물 별(star). 한 사용자가 한 게시물에 한 번만
CREATE TABLE post_stars (
    post_id     BIGINT      NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id     BIGINT      NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (post_id, user_id)
);

ALTER TABLE posts    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT now();
ALTER TABLE comments ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT now();
