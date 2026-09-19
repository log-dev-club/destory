-- 게시물 본문(마크다운)에 삽입하는 인라인 이미지.
-- post_attachments(릴리즈 파일)와 달리 작성 중(게시물 저장 전)에도 업로드해야 하므로 post_id 를 두지 않고,
-- 디스크에는 무작위 stored_name 으로만 저장한다. 공개 URL(APP_BASE_URL 기반)로만 참조되므로 다운로드 시
-- 원본 파일명을 보존할 필요가 없다.
CREATE TABLE content_images (
    id          BIGSERIAL    PRIMARY KEY,
    uploader_id BIGINT       NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stored_name VARCHAR(80)  NOT NULL UNIQUE,
    size_bytes  BIGINT       NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_content_images_uploader_id ON content_images (uploader_id);
