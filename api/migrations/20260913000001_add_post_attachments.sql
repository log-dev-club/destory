-- 게시물 작성 중 첨부한 릴리즈 파일(zip, exe 등)
-- 저장/다운로드는 hashed_name(전송 시각 + 원본 파일명을 SHA-256 해싱한 값)만으로 참조하고,
-- 사용자에게 보여줄 이름은 original_name에 별도로 보관한다.
CREATE TABLE post_attachments (
    id            BIGSERIAL    PRIMARY KEY,
    post_id       BIGINT       NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    original_name VARCHAR(255) NOT NULL,
    hashed_name   VARCHAR(80)  NOT NULL UNIQUE,  -- sha256(sent_at + original_name) + 확장자
    size_bytes    BIGINT       NOT NULL,
    sent_at       TIMESTAMPTZ  NOT NULL,         -- 해싱에 사용된 전송 시각
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_post_attachments_post_id ON post_attachments (post_id);
