-- 사용자 역할 (일반/관리자/최고 관리자).
-- 최고 관리자는 다른 계정에 관리자 권한을 부여/해제할 수 있고, 관리자는 게시물·댓글을 작성자와 무관하게 삭제할 수 있다.
ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'user';
ALTER TABLE users ADD CONSTRAINT users_role_check CHECK (role IN ('user', 'admin', 'super_admin'));

CREATE INDEX idx_users_role ON users (role) WHERE role <> 'user';
