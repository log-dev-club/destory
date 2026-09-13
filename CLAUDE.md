# CLAUDE.md

이 파일은 Claude Code가 이 저장소에서 작업할 때 참고하는 팀 공용 가이드입니다.

## 프로젝트 개요

**destory** — 팀 내 프로젝트 및 학습 정리 문서를 게시물로 올리고 공유하는 게시판 서비스.

- 디자인: GeekNews 스타일, 블랙(다크) 테마 고정
- 언어: 코드 주석·커밋 메시지 본문·문서는 **한국어**로 작성 (식별자는 영어)

## 저장소 구조 (모노레포)

```
destory/
├── api/                    # Rust 백엔드 (axum + sqlx + tokio)
│   ├── src/                # 모듈 구성은 아래 "Backend 모듈 구성" 참고
│   ├── Dockerfile          # 릴리즈 빌드 → 경량 런타임 이미지
│   ├── migrations/         # sqlx 마이그레이션 SQL (파일명: YYYYMMDDHHMMSS_설명.sql)
│   ├── uploads/            # 첨부파일 저장소 (gitignore, UPLOAD_DIR 로 변경 가능)
│   ├── Cargo.toml
│   ├── rustfmt.toml        # 포맷 규칙 (max_width=100, edition 2024)
│   └── .env.example        # 환경 변수 템플릿 (.env 는 커밋 금지)
├── frontend/               # React 19 + TypeScript + Vite 8 + react-router 7
│   ├── src/main.tsx        # 진입점
│   ├── src/App.tsx         # 라우팅 (/, /write, /me, /posts/:id)
│   ├── src/types.ts        # API 응답 타입 (백엔드 DTO 와 필드명 일치)
│   ├── src/api/            # fetch 래퍼 (도메인별)
│   ├── vite.config.ts      # /api → http://127.0.0.1:8080 프록시
│   ├── Dockerfile          # Vite 빌드 → Caddy 이미지 (정적 서빙 + /api 프록시 + HTTPS)
│   ├── Caddyfile
│   └── eslint.config.js
├── docs/api.md             # REST API 상세 명세 (요청/응답 예시)
├── docs/deploy.md          # Docker 배포 절차
├── docker-compose.yml      # db + api + web(Caddy) 전체 스택
├── .env.example            # docker compose 용 환경 변수 템플릿 (로컬 개발은 api/.env)
├── .gitignore
└── CLAUDE.md
```

## 기술 스택

| 영역 | 스택 |
|---|---|
| Backend | Rust (edition 2024), axum 0.8 (+axum-extra cookie), tokio, sqlx 0.8, serde, tracing, argon2, reqwest |
| Frontend | React 19, TypeScript 5.9 (strict), Vite 8, react-router-dom 7, react-markdown + remark-gfm + rehype-highlight |
| DB | PostgreSQL (sqlx `postgres` feature). 서버 기동 시 DB 가 없으면 생성하고 마이그레이션을 자동 적용 |
| 인증 | 닉네임 + 비밀번호(argon2) 또는 GitHub OAuth. 세션 토큰을 HttpOnly 쿠키(`session`)로 전달 |
| 외부 연동 | Discord Webhook (게시물 업로드 알림) |

## 자주 쓰는 명령어

### Backend (`api/`)
```bash
cd api
cp .env.example .env      # 최초 1회, DATABASE_URL 의 비밀번호만 본인 것으로 수정
cargo check               # 빠른 타입 검사 (바이너리 생성 없음)
cargo run                 # 서버 실행 → http://127.0.0.1:8080/api/ping (DB 없으면 자동 생성, 마이그레이션 자동 적용)
cargo fmt                 # 코드 작성 완료 후 반드시 실행
cargo clippy              # 린트
```

### 환경 변수 (`api/.env`)
| 키 | 기본값 | 설명 |
|---|---|---|
| `DATABASE_URL` | (필수) | `postgres://user:pw@localhost:5432/destory` |
| `SERVER_PORT` | 8080 | 바인딩 포트 |
| `RUST_LOG` | info | tracing 레벨 |
| `DISCORD_WEBHOOK_URL` | 없음 | 미설정 또는 예시값이면 알림 생략 |
| `UPLOAD_DIR` | ./uploads | 첨부파일 저장 경로 |
| `MAX_UPLOAD_MB` | 100 | 첨부 업로드 본문 한도 |
| `SESSION_TTL_DAYS` | 30 | 세션 쿠키 만료 |
| `APP_BASE_URL` | http://localhost:5173 | Discord 알림 링크의 기준 URL |
| `GITHUB_CLIENT_ID` / `GITHUB_CLIENT_SECRET` | 없음 | GitHub OAuth App. 둘 다 있어야 GitHub 로그인 활성화 |
| `GITHUB_REDIRECT_URL` | {APP_BASE_URL}/api/auth/github/callback | OAuth App 의 callback URL 과 동일해야 함 |
| `AUTH_RATE_LIMIT_PER_MINUTE` | 10 | 로그인·회원가입·GitHub 시작의 IP 당 분당 요청 한도 |
| `TRUST_PROXY_HEADERS` | false | 리버스 프록시 뒤에서만 true. X-Forwarded-For 의 IP 를 신뢰 |
| `SERVER_HOST` | 127.0.0.1 | 바인딩 주소. Docker 에서는 0.0.0.0 |
| `COOKIE_SECURE` | false | HTTPS 배포에서 true. 세션 쿠키 Secure 속성 |

### Frontend (`frontend/`)
```bash
cd frontend
npm install
npm run dev               # Vite 개발 서버
npm run build             # tsc -b && vite build
npm run lint              # eslint .
```

### `git pull` 이후 체크리스트
받아온 변경에 따라 아래를 수행한다. 실행 중인 서버는 새 의존성·마이그레이션을 자동으로 인식하지 못한다.

| 바뀐 파일 | 해야 할 것 |
|---|---|
| `frontend/package.json` | `npm install` 후 **`npm run dev` 재시작** (Vite 는 기동 시점의 패키지 목록을 캐시하므로 재시작 없이는 `Failed to resolve import` 오류가 남) |
| `api/Cargo.toml` | `cargo build` 후 서버 재시작 |
| `api/migrations/*.sql` | 서버 재시작 (기동 시 자동 적용). 기존 파일이 수정된 경우에만 DB 재생성 |
| `api/.env.example` | 새 키를 본인 `.env` 에 추가 (대부분 비워도 기본값으로 동작) |

### 배포 (Docker)
```bash
cp .env.example .env      # POSTGRES_PASSWORD, SITE_ADDRESS, APP_BASE_URL 채우기
docker compose up -d --build
docker compose logs -f api
```
클라우드 VM 이든 개인 PC 든 동일. 상세 절차와 주소 설정은 [docs/deploy.md](docs/deploy.md).

## 아키텍처

### 요청 흐름
```
브라우저 → Vite dev server(/api 프록시) → axum(127.0.0.1:SERVER_PORT) → PostgreSQL
                                              └→ Discord Webhook (게시물 업로드 시)
```

### Backend 모듈 구성 (기능 추가 시 이 구조를 따른다)
```
api/src/
├── main.rs          # 부트스트랩만 담당. 비즈니스 로직 작성 금지
├── config.rs        # .env → Config 구조체. 환경 변수는 여기서만 읽는다
├── state.rs         # AppState { pool, config, http }. 모든 핸들러가 State<AppState> 로 받음
├── error.rs         # AppError → { "error": msg } + 상태 코드. sqlx/io/multipart 에러 From 변환 포함
├── extract.rs       # CurrentUser(로그인 필수, 401) / MaybeUser(선택) 추출기
├── rate_limit.rs    # IP 당 요청 횟수 제한 레이어 (tower_governor). 무차별 대입 대상 라우터에만 적용
├── routes/          # 도메인별 Router<AppState> (auth, users, posts, comments, attachments, tags)
├── handlers/        # axum 핸들러. 요청 파싱 → 서비스 호출 → 응답 변환. DB 쿼리 작성 금지
├── services/        # 비즈니스 로직 + DB 접근 (검증, 트랜잭션, 첨부파일 해시 검증, Discord 알림)
└── models/          # DB row 구조체 (sqlx::FromRow) 및 요청/응답 DTO (serde, camelCase)
```
- 새 도메인을 추가할 때는 `models/ → services/ → handlers/ → routes/` 순서로 파일을 만들고 `routes/mod.rs` 에서 `.merge()` 한다.
- 인증이 필요한 핸들러는 인자로 `CurrentUser(user)` 를 받는다. 미들웨어로 경로별 인증을 걸지 않는다.
- 소유권 검사(작성자만 수정/삭제)는 서비스 계층에서 `AppError::Forbidden` 으로 처리한다.
- 쿼리는 `sqlx::query_as` 런타임 API 를 사용한다. `query!` 매크로는 컴파일 시 DB 연결이 필요하므로 사용하지 않는다.
- 동적 WHERE 는 `sqlx::QueryBuilder` + `push_bind` 로 만든다. 문자열 포맷으로 값을 넣지 않는다.

### Frontend 모듈 구성
```
frontend/src/
├── main.tsx
├── App.tsx          # BrowserRouter + Layout. 라우팅만 담당
├── types.ts         # API 응답 타입. 백엔드 DTO 와 필드명(camelCase) 일치
├── index.css        # 테마 변수 (색상 추가는 여기서만)
├── components/      # Header, SearchBar, PostCard, Markdown, Layout ...
├── pages/           # HomePage, PostDetailPage, WritePage, MyPage
├── utils/           # searchQuery (tag:/user: 파싱), hashFile (첨부파일 해시)
└── api/             # fetch 래퍼 (도메인별 파일). 컴포넌트에서 fetch 직접 호출 금지
```
- 검색창 입력은 `parseSearchQuery` → `toPostListParams` 로 `tag`(쉼표 연결), `user`, `q`(공백 연결) 쿼리 파라미터를 만들어 API 에 넘긴다. 클라이언트에서 필터링하지 않는다.
- 첨부파일은 `buildHashedRelease` 로 얻은 `hashedName`, `sentAt` 을 파일과 함께 multipart 로 보낸다. 서버가 같은 방식으로 재계산해 검증하므로 값을 임의로 바꾸면 400 이 난다.
- 인증은 쿠키 기반이라 same-origin 프록시 환경에서는 `fetch` 에 별도 설정이 필요 없다. 401 응답이면 로그인 화면으로 보낸다.

### DB 스키마
| 테이블 | 마이그레이션 | 용도 | 주요 컬럼 |
|---|---|---|---|
| `users` | `..init`, `..20260914000000`, `..20260914100000` | 사용자 | `nickname` (UNIQUE), `password_hash` (argon2, GitHub 가입은 빈 문자열), `avatar_url`, `bio`, `github_id` (UNIQUE), `github_login` |
| `sessions` | `20260914000000_add_auth_stars_profile.sql` | 로그인 세션 | `token` (PK, 64 hex), `user_id`, `expires_at` |
| `posts` | `20260913000000_init.sql` | 게시물 | `title`, `summary` (excerpt), `content` (GFM 원문), `created_at`, `updated_at` |
| `tags` / `post_tags` | `20260913000000_init.sql` | 태그 (N:M) | `name` (UNIQUE, 소문자 정규화) |
| `comments` | `20260913000000_init.sql` | 댓글 | `post_id`, `user_id`, `content`, `updated_at` |
| `post_stars` | `20260914000000_add_auth_stars_profile.sql` | 별 | PK(`post_id`, `user_id`) |
| `post_attachments` | `20260913000001_add_post_attachments.sql` | 첨부 릴리즈 파일 | `post_id`, `original_name`, `hashed_name` (UNIQUE, 디스크 파일명), `size_bytes`, `sent_at` |

- 데이터베이스 생성은 마이그레이션이 아니라 `main.rs` 시작 단계(`Postgres::create_database`)에서 한다. 마이그레이션 SQL 은 이미 접속한 DB 안에서 실행되므로 `CREATE DATABASE` 를 넣을 수 없다.
- 스키마 변경은 기존 파일을 수정하지 말고 **새 마이그레이션 파일을 추가**한다. 이미 적용된 마이그레이션은 절대 수정하지 않는다. (수정하면 서버 기동 시 체크섬 불일치로 실패하며, 이미 적용한 팀원은 DB 를 지우고 다시 만들어야 한다.)
- 파일명은 `YYYYMMDDHHMMSS_설명.sql` 형식 (타임스탬프 순서로 실행됨).
- `post_attachments.hashed_name` 은 프론트엔드가 `SHA-256("{sentAt}:{originalName}")` + 확장자로 계산한 값이다. 서버는 업로드 시 같은 값을 재계산해 다르면 거부하고, 디스크에는 이 이름으로만 저장한다. 다운로드 시 `Content-Disposition` 에 `original_name` 을 넣어 준다.

### API 규약
- 모든 엔드포인트는 `/api` prefix. 리소스 복수형 명사 사용 (`/api/posts`, `/api/posts/{id}/comments`).
- 요청/응답 JSON 필드는 **camelCase** (프론트엔드 `types.ts` 기준). Rust DTO 는 `#[serde(rename_all = "camelCase")]`, DB 컬럼은 snake_case 유지.
- 날짜는 RFC 3339 문자열 (`createdAt`, `updatedAt`). 첨부파일 `sentAt` 만 밀리초 epoch 숫자 (JS `Date.now()` 와 동일).
- `id` 는 숫자(i64). 프론트 `Post.id: string` 은 연동 시 `number` 로 바꾼다.
- 목록은 `{ items, page, limit, total }` 페이지 형태. `page` 는 1부터, `limit` 기본 20·최대 100.
- 목록 항목(`PostSummary`)에는 `content` 가 없고 상세(`PostDetail`)에만 있다. 상세에는 `attachments` 배열이 포함된다.
- 검색: `?tag=react,frontend` (모두 포함, 부분 일치) · `?user=닉네임` (부분 일치) · `?q=단어1 단어2` (제목·요약·닉네임·태그 중 모두 포함).
- 인증 필요 요청에 세션이 없으면 401, 남의 리소스를 수정하면 403, 없는 리소스는 404, 닉네임/해시 중복은 409, 요청 횟수 초과는 429 (`Retry-After` 헤더 포함).
- 에러 응답 형식: `{ "error": "<메시지>" }` + HTTP 상태 코드. 메시지는 사용자에게 그대로 보여줄 수 있는 한국어.
- 요청/응답 예시는 [docs/api.md](docs/api.md) 참고. 엔드포인트를 추가·변경하면 그 문서도 함께 갱신한다.

### 엔드포인트 요약
| 메서드 | 경로 | 인증 | 설명 |
|---|---|---|---|
| POST | `/api/auth/register` | – | 회원가입 (닉네임 2~20자, 비밀번호 8자+ 특수문자 포함) → 세션 쿠키 발급 |
| POST | `/api/auth/login` | – | 로그인 → 세션 쿠키 발급 |
| POST | `/api/auth/logout` | 필요 | 세션 삭제 |
| GET | `/api/auth/me` | 필요 | 현재 사용자 프로필 |
| GET | `/api/auth/providers` | – | 활성화된 외부 로그인 (`{ github }`) |
| GET | `/api/auth/github` | – | GitHub 인가 화면으로 302 (로그인 상태면 계정 연동) |
| GET | `/api/auth/github/callback` | – | GitHub 콜백 → 세션 발급 후 `/` 302, 실패 시 `/login?error=` |
| GET / PATCH | `/api/users/me` | 필요 | 마이페이지 프로필 조회 / 수정 (`nickname`, `avatarUrl`, `bio`) |
| PUT | `/api/users/me/password` | 필요 | 비밀번호 변경 (다른 세션 전부 만료) |
| GET | `/api/users/me/posts` | 필요 | 내가 쓴 게시물 (검색 파라미터 동일) |
| GET | `/api/users/{nickname}` | – | 공개 프로필 |
| GET | `/api/users/{nickname}/posts` | – | 특정 사용자의 게시물 |
| GET | `/api/posts` | 선택 | 최신순 목록 + 검색/필터/페이지 |
| POST | `/api/posts` | 필요 | 게시물 작성 → Discord 알림 |
| GET | `/api/posts/{id}` | 선택 | 상세 (content, attachments, starred 포함) |
| PATCH / DELETE | `/api/posts/{id}` | 작성자 | 수정 (`tags` 는 전체 교체) / 삭제 (첨부파일도 삭제) |
| PUT / DELETE | `/api/posts/{id}/star` | 필요 | 별 추가 / 해제 → `{ starred, starCount }` |
| GET / POST | `/api/posts/{id}/comments` | – / 필요 | 댓글 목록 (오래된 순) / 작성 |
| PATCH / DELETE | `/api/comments/{id}` | 작성자 / 작성자·글쓴이 | 댓글 수정 / 삭제 |
| GET / POST | `/api/posts/{id}/attachments` | – / 작성자 | 첨부 목록 / 업로드 (multipart: `file`, `hashedName`, `sentAt`) |
| GET / DELETE | `/api/attachments/{hashedName}` | – / 작성자 | 다운로드 (원본 파일명) / 삭제 |
| GET | `/api/tags` | – | 사용 중인 태그와 게시물 수 |

## 기능 명세

### Header
- 검색창: `tag:react user:yangmin 자유텍스트` 형식. `tag:`/`user:` 접두어는 필터, 나머지는 제목·요약·닉네임·태그 검색
- 마이페이지(`/me`): 프로필(닉네임, 아바타, 소개) 조회·수정, 내가 올린 게시물 목록, 비밀번호 변경

### Main
- 최신 게시물 순(`created_at DESC`)으로 표시. 카드에 작성자, 날짜, 태그, 별 수, 댓글 수
- 게시물별 댓글 (작성·수정·삭제). 삭제는 댓글 작성자 또는 게시물 작성자
- 별(star): 로그인 사용자당 게시물 하나에 한 번

### 게시물 (Post)
- 프로젝트 / 학습 정리 문서 업로드
- **본문은 GitHub 스타일 Markdown(GFM)** 으로 작성·저장·렌더링
  - 지원 문법: 제목, 목록, 체크리스트, 표, 코드 블록(언어별 하이라이트), 인용, 링크, 이미지, 취소선
  - DB에는 Markdown 원문을 그대로 저장하고, 렌더링은 프론트엔드에서 수행
  - 렌더링 시 XSS 방지를 위해 HTML은 sanitize 하고, raw HTML 태그는 허용하지 않음
  - 작성 화면은 에디터 + 미리보기(preview) 형태 권장
- 릴리즈 파일(zip, exe 등) 첨부 시 **전송 시각 + 원본 파일명을 해싱한 값**을 파일명으로 하여 서버/DB에 전달 (`post_attachments.hashed_name`)
  - DB는 해싱된 이름만으로 파일을 저장·다운로드하고, 사용자에게 보여줄 원본 파일명은 `original_name`에 별도로 보관
- 업로드 시 **Discord로 제목 + 간략 설명 전송** (Webhook). 본문의 첫 이미지(공개 http(s) URL 만)를 썸네일로, 작성자 아바타를 아이콘으로 포함
  - Discord 전송 실패가 게시물 저장 자체를 실패시키면 안 됨 (로그만 남기고 진행)
  - Webhook URL은 `.env`의 `DISCORD_WEBHOOK_URL`에서 읽음 (하드코딩 금지)

## 코딩 규칙

### Backend
- **코드 작성이 끝나면 마지막에 반드시 `cargo fmt` 를 실행**한다. 포맷 규칙은 `api/rustfmt.toml`을 따른다.
- `cargo check`가 통과하는 상태로 커밋한다.
- 라우트는 `/api/...` prefix를 사용한다.
- 사용자 입력 검증(길이, 형식)은 서비스 계층 진입부에서 하고 `AppError::BadRequest` 로 돌려준다. DB 제약에만 의존하지 않는다.
- 비밀번호 해시 등 CPU 작업은 `tokio::task::spawn_blocking` 으로 감싼다.
- 무차별 대입 대상이 되는 엔드포인트(로그인, 회원가입 등)를 추가하면 `routes/auth.rs` 처럼 `rate_limit::limit_per_ip` 로 감싼다. 일반 조회 API 에는 걸지 않는다.
- 외부 로그인(OAuth)은 `services/github.rs` 패턴을 따른다: 인가 URL 생성 → state 쿠키 → 콜백에서 토큰 교환·사용자 조회 → `find_or_create`/`link`. 콜백은 브라우저 리다이렉트이므로 오류를 JSON 이 아니라 `/login?error=` 로 돌려보낸다.
- 비밀번호 규칙은 백엔드 `services/auth.rs::validate_password` 와 프론트 `utils/password.ts` 두 곳에 있다. 바꿀 때 둘 다 수정한다.
- DB 접근은 sqlx를 사용하고, 쿼리는 PostgreSQL 문법(`$1, $2` 바인딩)으로 작성한다.
- 환경 변수는 `dotenvy` + `std::env`로 읽는다. 새 변수를 추가하면 `api/.env.example`에 설명과 함께 반드시 추가한다.
- 에러는 `expect`/`unwrap`으로 패닉시키지 말고 핸들러에서는 적절한 HTTP 상태 코드로 변환한다. (초기화 단계인 `main.rs`의 `expect`는 예외)
- 로깅은 `tracing` 매크로(`info!`, `warn!`, `error!`)를 사용한다. `println!` 금지.

### Frontend
- TypeScript strict 모드 유지 (`noUnusedLocals`, `noUnusedParameters` 활성화됨). `any` 사용 금지.
- `verbatimModuleSyntax`가 켜져 있으므로 타입 import는 `import type { ... }` 로 작성한다.
- 다크 테마 고정: 시스템 테마에 따라 라이트로 바뀌지 않도록 한다. 색상은 CSS 변수로 관리한다.
- GeekNews 스타일: 텍스트 중심의 밀도 높은 리스트, 최소한의 장식, 좁은 여백.
- Markdown 렌더링은 GFM 호환 라이브러리(예: `react-markdown` + `remark-gfm`)를 사용하고, 코드 블록은 다크 테마용 하이라이트 스타일을 적용한다. 새 라이브러리 도입 시 GFM 표·체크리스트 지원 여부를 확인한다.
- API 호출은 상대 경로 `/api/...`로 작성한다 (Vite 프록시가 백엔드로 전달). 절대 URL 하드코딩 금지.
- 커밋 전 `npm run lint` 와 `npm run build` 가 통과해야 한다.

### 공통
- `.env` 파일은 절대 커밋하지 않는다 (`.gitignore`에 포함됨). 시크릿이 필요하면 `.env.example`에 키 이름만 추가한다.
- 주석은 한국어로 작성한다. 팀원에게 안내가 필요한 부분은 `// 팀원 참고:` 형식을 사용한다.
- 줄바꿈은 **LF**로 통일한다 (`.gitattributes`로 강제). Windows에서 `LF will be replaced by CRLF` 경고가 보이면 `git config core.autocrlf false` 로 끄고 `git checkout -- .` 으로 다시 받는다. `rustfmt.toml`의 `newline_style = "Unix"`와 일치시키기 위함.

## Git 컨벤션

- 기본 브랜치: `main`. 개인 작업 브랜치: `<이니셜>/dev` (예: `jh/dev`) → `main`으로 PR.
- 커밋 메시지: `<type>: <설명>` 형식. type은 `feat`, `fix`, `chore`, `refactor`, `docs`, `style`, `test` 중 하나.
  - 예: `feat: 게시물 업로드 API 추가`, `chore: translate comments to korean`
