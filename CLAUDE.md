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
│   ├── src/main.rs         # 진입점: .env 로드 → 로깅 → DB 풀 → 마이그레이션 → 라우터 → 서버
│   ├── migrations/         # sqlx 마이그레이션 SQL (파일명: YYYYMMDDHHMMSS_설명.sql)
│   ├── Cargo.toml
│   ├── rustfmt.toml        # 포맷 규칙 (max_width=100, edition 2024)
│   └── .env.example        # 환경 변수 템플릿 (.env 는 커밋 금지)
├── frontend/               # React 19 + TypeScript + Vite 8
│   ├── src/main.tsx        # 진입점
│   ├── src/App.tsx         # 레이아웃 골격 (헤더 + 게시물 목록)
│   ├── src/index.css       # 블랙 테마 CSS 변수 및 전역 스타일
│   ├── vite.config.ts      # /api → http://127.0.0.1:8080 프록시
│   └── eslint.config.js
├── .gitignore
└── CLAUDE.md
```

## 기술 스택

| 영역 | 스택 |
|---|---|
| Backend | Rust (edition 2024), axum 0.8, tokio, sqlx 0.8, serde, tracing, dotenvy |
| Frontend | React 19, TypeScript 5.9 (strict), Vite 8, ESLint 9 (flat config) |
| DB | PostgreSQL (sqlx `postgres` feature, 마이그레이션은 `sqlx::migrate!` 로 서버 기동 시 자동 적용) |
| 외부 연동 | Discord Webhook (게시물 업로드 알림) |

## 자주 쓰는 명령어

### Backend (`api/`)
```bash
cd api
cp .env.example .env      # 최초 1회, DATABASE_URL 채우기
cargo check               # 빠른 타입 검사 (바이너리 생성 없음)
cargo run                 # 서버 실행 → http://127.0.0.1:8080/api/ping
cargo fmt                 # 코드 작성 완료 후 반드시 실행
cargo clippy              # 린트
```

### Frontend (`frontend/`)
```bash
cd frontend
npm install
npm run dev               # Vite 개발 서버
npm run build             # tsc -b && vite build
npm run lint              # eslint .
```

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
├── routes/          # 도메인별 라우터 (posts.rs, comments.rs, users.rs ...)
├── handlers/        # axum 핸들러. 요청 파싱 → 서비스 호출 → 응답 변환
├── services/        # 비즈니스 로직 (첨부파일 해싱, Discord 알림 등)
├── models/          # DB row 구조체 (sqlx::FromRow) 및 요청/응답 DTO (serde)
└── error.rs         # 공통 에러 타입 → IntoResponse 구현
```
- 라우터는 도메인별 파일에서 `Router<PgPool>`을 반환하고 `main.rs`에서 `.merge()` 또는 `.nest("/api/...")` 로 합친다.
- 핸들러는 `State<PgPool>`로 커넥션 풀을 받는다. 전역 static 사용 금지.

### Frontend 모듈 구성
```
frontend/src/
├── main.tsx
├── App.tsx          # 라우팅과 레이아웃만 담당
├── index.css        # 테마 변수 (색상 추가는 여기서만)
├── components/      # 재사용 UI (Header, PostList, PostItem, CommentList, MarkdownViewer ...)
├── pages/           # 화면 단위 (Home, PostDetail, PostWrite, MyPage)
├── api/             # fetch 래퍼. 컴포넌트에서 fetch 직접 호출 금지
└── types/           # API 응답 타입 (백엔드 DTO와 필드명 일치)
```

### DB 스키마
| 테이블 | 마이그레이션 | 용도 | 주요 컬럼 |
|---|---|---|---|
| `users` | `20260913000000_init.sql` | 작성자 | `user_name` (UNIQUE, 헤더 필터 키) |
| `posts` | `20260913000000_init.sql` | 게시물 | `title`, `summary`, `content` (GFM 원문), `created_at` |
| `tags` / `post_tags` | `20260913000000_init.sql` | 태그 (N:M) | `name` (UNIQUE) |
| `comments` | `20260913000000_init.sql` | 댓글 | `post_id`, `user_id`, `content` |
| `post_attachments` | `20260913000001_add_post_attachments.sql` | 게시물에 첨부한 릴리즈 파일 (zip, exe 등) | `post_id`, `original_name`, `hashed_name` (SHA-256(`sent_at`+원본 파일명), UNIQUE, 다운로드 참조 키), `size_bytes`, `sent_at` |

- 스키마 변경은 기존 파일을 수정하지 말고 **새 마이그레이션 파일을 추가**한다. 이미 적용된 마이그레이션은 절대 수정하지 않는다.
- 파일명은 `YYYYMMDDHHMMSS_설명.sql` 형식 (타임스탬프 순서로 실행됨).
- `post_attachments.hashed_name`은 프론트엔드가 `crypto.subtle.digest('SHA-256', sentAt:originalName)`로 계산한 값과 확장자를 합친 문자열이다. 서버/DB는 이 해시값만으로 파일을 저장·다운로드하며, 사용자에게 보여줄 이름은 `original_name`에서 가져온다.

### API 규약
- 모든 엔드포인트는 `/api` prefix. 리소스 복수형 명사 사용 (`/api/posts`, `/api/posts/{id}/comments`).
- 요청/응답 JSON 필드는 `snake_case`. 프론트 타입도 동일하게 맞춘다.
- 목록 조회는 `?tag=`, `?user_name=`, `?q=` 쿼리 파라미터로 필터·검색을 받는다.
- 에러 응답 형식: `{ "error": "<메시지>" }` + 적절한 HTTP 상태 코드.

## 기능 명세

### Header
- 검색창 (게시물 제목·본문 검색)
- 필터: `tag`, `user_name`
- 마이페이지: 자신이 올린 게시물 목록 확인

### Main
- 최신 게시물 순(`created_at DESC`)으로 표시
- 게시물별 댓글

### 게시물 (Post)
- 프로젝트 / 학습 정리 문서 업로드
- **본문은 GitHub 스타일 Markdown(GFM)** 으로 작성·저장·렌더링
  - 지원 문법: 제목, 목록, 체크리스트, 표, 코드 블록(언어별 하이라이트), 인용, 링크, 이미지, 취소선
  - DB에는 Markdown 원문을 그대로 저장하고, 렌더링은 프론트엔드에서 수행
  - 렌더링 시 XSS 방지를 위해 HTML은 sanitize 하고, raw HTML 태그는 허용하지 않음
  - 작성 화면은 에디터 + 미리보기(preview) 형태 권장
- 릴리즈 파일(zip, exe 등) 첨부 시 **전송 시각 + 원본 파일명을 해싱한 값**을 파일명으로 하여 서버/DB에 전달 (`post_attachments.hashed_name`)
  - DB는 해싱된 이름만으로 파일을 저장·다운로드하고, 사용자에게 보여줄 원본 파일명은 `original_name`에 별도로 보관
- 업로드 시 **Discord로 제목 + 간략 설명 전송** (Webhook)
  - Discord 전송 실패가 게시물 저장 자체를 실패시키면 안 됨 (로그만 남기고 진행)
  - Webhook URL은 `.env`의 `DISCORD_WEBHOOK_URL`에서 읽음 (하드코딩 금지)

## 코딩 규칙

### Backend
- **코드 작성이 끝나면 마지막에 반드시 `cargo fmt` 를 실행**한다. 포맷 규칙은 `api/rustfmt.toml`을 따른다.
- `cargo check`가 통과하는 상태로 커밋한다.
- 라우트는 `/api/...` prefix를 사용한다.
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
