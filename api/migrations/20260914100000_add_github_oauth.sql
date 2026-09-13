-- GitHub OAuth 로그인: users 에 GitHub 계정 식별자 추가
-- github_id 로 가입한 사용자는 password_hash 가 빈 문자열이라 비밀번호 로그인은 불가
ALTER TABLE users
    ADD COLUMN github_id    BIGINT UNIQUE,     -- GitHub 사용자 고유 id (login 은 바뀔 수 있으므로 id 로 매칭)
    ADD COLUMN github_login VARCHAR(100);      -- 화면 표시용 GitHub 로그인명
