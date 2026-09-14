# destory REST API

모든 경로는 `/api` 아래에 있고, 요청/응답 본문은 JSON(camelCase)입니다. 인증은 `session` HttpOnly 쿠키로 이루어지며, 로그인/회원가입 응답에서 자동으로 설정됩니다.

공통 에러 응답:

```json
{ "error": "사용자에게 보여줄 수 있는 메시지" }
```

| 상태 | 의미 |
|---|---|
| 400 | 입력 검증 실패 (메시지에 이유) |
| 401 | 로그인 필요 |
| 403 | 본인 리소스가 아님 |
| 404 | 리소스 없음 |
| 409 | 중복 (닉네임, 첨부파일 hashedName) |
| 413 | 첨부파일 크기 초과 (`MAX_UPLOAD_MB`) |
| 429 | 요청 횟수 초과. `Retry-After` 헤더(초)에 재시도 가능 시각. 로그인·회원가입·GitHub 시작에만 적용 (IP 당 분당 `AUTH_RATE_LIMIT_PER_MINUTE`, 기본 10) |

## 공통 타입

```ts
interface Author { nickname: string; avatarUrl?: string }

interface PostSummary {
  id: number
  title: string
  excerpt: string
  author: Author
  tags: string[]
  createdAt: string      // RFC 3339
  updatedAt: string
  commentCount: number
  starCount: number
  starred: boolean       // 요청자가 별을 눌렀는지 (비로그인 false)
}

interface PostDetail extends PostSummary {
  content: string        // GFM Markdown 원문
  attachments: Attachment[]
}

interface PostPage { items: PostSummary[]; page: number; limit: number; total: number }

interface Comment {
  id: number; postId: number; content: string; author: Author
  createdAt: string; updatedAt: string
}

interface Attachment {
  id: number; postId: number
  originalName: string   // 사용자에게 보여줄 이름
  hashedName: string     // 디스크 파일명 = sha256(`${sentAt}:${originalName}`) + 확장자
  size: number           // bytes
  sentAt: number         // ms epoch (Date.now())
  downloadUrl: string    // "/api/attachments/{hashedName}"
  createdAt: string
}

interface UserProfile {
  id: number; nickname: string; avatarUrl?: string; bio?: string
  githubLogin?: string   // GitHub 로 가입/연동한 경우에만 존재
  createdAt: string
  postCount: number      // 작성한 게시물 수
  starCount: number      // 작성한 게시물이 받은 별 합계
}

interface TagCount { name: string; postCount: number }
```

## 인증

### POST /api/auth/register
```json
{ "nickname": "alice", "password": "pass!word1" }
```
- 닉네임: 2~20자, 문자·숫자·`_`·`-`. 비밀번호: 8자 이상, 공백 없음, 특수문자(문자·숫자가 아닌 문자) 1개 이상.
- 비밀번호 확인 일치 여부는 프론트엔드에서 검사한다 (서버는 `password` 하나만 받음).
- 201 + `UserProfile`, `session` 쿠키 설정. 중복 닉네임은 409.

### POST /api/auth/login
```json
{ "nickname": "alice", "password": "pass!word1" }
```
- 200 + `UserProfile`, `session` 쿠키 설정. 실패는 400 (닉네임/비밀번호 구분 없음).

### POST /api/auth/logout
- 204. 쿠키 제거.

### GET /api/auth/me
- 200 + `UserProfile`. 비로그인 401.

### GET /api/auth/providers
```json
{ "github": true }
```
- 서버에 설정된 외부 로그인 제공자. `false` 면 프론트는 버튼을 숨긴다.

### GET /api/auth/github
- 브라우저를 GitHub 인가 화면으로 302 리다이렉트한다. `fetch` 가 아니라 `window.location.href = '/api/auth/github'` 로 호출한다.
- CSRF 방지용 `state` 를 10분짜리 HttpOnly 쿠키(`gh_oauth_state`)에 저장한다.
- `GITHUB_CLIENT_ID`/`GITHUB_CLIENT_SECRET` 미설정이면 404.

### GET /api/auth/github/callback?code=&state=
- GitHub 가 호출하는 콜백. 직접 호출하지 않는다.
- 성공: `session` 쿠키 발급 후 `/` 로 302. **이미 로그인한 상태**였으면 현재 계정에 GitHub 를 연동하고 `/me` 로 302.
- 실패(취소, state 불일치, 토큰 교환 실패 등): `/login?error=<메시지>` 로 302. JSON 에러를 돌려주지 않는다.
- 처음 로그인하는 GitHub 계정은 사용자를 자동 생성한다. 닉네임은 GitHub login 을 닉네임 규칙에 맞게 다듬고 중복이면 `-2`, `-3` 을 붙인다. 아바타는 GitHub 프로필 사진. 비밀번호가 없으므로 비밀번호 로그인·변경은 불가.
- 같은 GitHub 계정을 두 사용자에 연동할 수 없다 (409 → `/login?error=`).

## 사용자

### GET /api/users/me
- 200 + `UserProfile` (마이페이지 상단).

### PATCH /api/users/me
```json
{ "nickname": "alice2", "avatarUrl": "https://example.com/a.png", "bio": "안녕하세요" }
```
- 모든 필드 선택. 생략한 필드는 유지. `avatarUrl`/`bio` 에 `""` 를 보내면 값을 지운다.
- `avatarUrl` 은 http(s) URL, 500자 이하. `bio` 는 300자 이하.
- 200 + `UserProfile`. 닉네임 중복 409.

### PUT /api/users/me/password
```json
{ "currentPassword": "pass!word1", "newPassword": "new#pass2" }
```
- 새 비밀번호도 회원가입과 같은 규칙. 204. 다른 기기의 세션은 모두 만료되고 현재 세션은 새 쿠키로 재발급된다. 현재 비밀번호 불일치 400.

### GET /api/users/me/posts
- `GET /api/posts` 와 같은 쿼리 파라미터를 받고 내 게시물만 돌려준다. 200 + `PostPage`.

### GET /api/users/me/draft
- 로그인 필요. 임시저장된 글이 있으면 200 + `Draft`, 없으면 200 + `null`.
```ts
interface Draft { title: string; content: string; tagsInput: string; updatedAt: string }
```

### PUT /api/users/me/draft
```json
{ "title": "쓰다 만 글", "content": "# 제목\n\n내용...", "tagsInput": "react, frontend" }
```
- 로그인 필요. 사용자당 1개, 있으면 덮어씀(upsert). 200 + `Draft`.

### DELETE /api/users/me/draft
- 로그인 필요. 임시저장 삭제(없어도 204).

### GET /api/users/{nickname}
- 200 + `UserProfile`. 없으면 404.

### GET /api/users/{nickname}/posts
- 200 + `PostPage`.

## 게시물

### GET /api/posts
쿼리 파라미터 (모두 선택):

| 파라미터 | 예 | 의미 |
|---|---|---|
| `tag` | `react,frontend` | 쉼표로 구분. 모든 태그가 부분 일치해야 함 |
| `user` | `yang` | 작성자 닉네임 부분 일치 |
| `q` | `useTransition 검색` | 공백으로 구분. 각 단어가 제목·요약·닉네임·태그 중 하나에 포함되어야 함 |
| `page` | `2` | 1부터. 기본 1 |
| `limit` | `20` | 기본 20, 최대 100 |

프론트엔드 `parseSearchQuery` 결과를 그대로 대응시키면 된다: `tags.join(',')` → `tag`, `users[0]` → `user`, `text.join(' ')` → `q`.

- 200 + `PostPage`. 최신순(`createdAt DESC`).

### POST /api/posts
```json
{
  "title": "React 19 useTransition 정리",
  "content": "# 문제\n\n대량 리스트 **필터링** ...",
  "excerpt": "입력 지연 없이 리스트를 필터링하는 방법",
  "tags": ["react", "frontend"]
}
```
- `title` 1~200자, `content` 필수. `excerpt` 생략 시 본문 앞 150자에서 자동 생성 (최대 300자).
- `tags` 는 소문자 정규화, 선행 `#` 제거, 중복 제거, 최대 10개·각 50자.
- 201 + `PostDetail`. 저장 후 Discord Webhook 으로 제목·요약·링크·태그를 embed 로 보낸다 (실패해도 응답에는 영향 없음). 본문의 첫 번째 마크다운 이미지(`![alt](https://...)`, 공개 URL 만)가 썸네일로, 작성자 아바타가 아이콘으로 붙는다.

### GET /api/posts/{id}
- 200 + `PostDetail`. 없으면 404.

### PATCH /api/posts/{id}
```json
{ "title": "수정된 제목", "tags": ["react", "hooks"] }
```
- 작성자만. 생략한 필드는 유지, `tags` 를 보내면 전체 교체. 200 + `PostDetail`.

### DELETE /api/posts/{id}
- 작성자만. 댓글·별·첨부파일(디스크 포함) 함께 삭제. 204.

### PUT /api/posts/{id}/star · DELETE /api/posts/{id}/star
- 로그인 필요. 멱등(두 번 눌러도 같은 결과).
```json
{ "starred": true, "starCount": 12 }
```

## 댓글

### GET /api/posts/{id}/comments
- 200 + `Comment[]` (오래된 순).

### POST /api/posts/{id}/comments
```json
{ "content": "좋은 글이네요" }
```
- 로그인 필요. 1~2000자. 201 + `Comment`.

### PATCH /api/comments/{id}
```json
{ "content": "수정된 댓글" }
```
- 댓글 작성자만. 200 + `Comment`.

### DELETE /api/comments/{id}
- 댓글 작성자 또는 게시물 작성자. 204.

## 첨부파일

### POST /api/posts/{id}/attachments
`multipart/form-data`, 게시물 작성자만. 요청당 파일 하나.

| 필드 | 값 |
|---|---|
| `file` | 파일 본문 (filename 필수) |
| `hashedName` | `buildHashedRelease(file).hashedName` |
| `sentAt` | `buildHashedRelease(file).sentAt` (ms epoch) |

```ts
const { hashedName, sentAt } = await buildHashedRelease(file)
const form = new FormData()
form.append('file', file)
form.append('hashedName', hashedName)
form.append('sentAt', String(sentAt))
await fetch(`/api/posts/${postId}/attachments`, { method: 'POST', body: form })
```
- 서버가 `sha256("{sentAt}:{file.name}") + 확장자` 를 재계산해 `hashedName` 과 다르면 400.
- 201 + `Attachment`. 같은 `hashedName` 이 이미 있으면 409. 크기 초과 413.

### GET /api/posts/{id}/attachments
- 200 + `Attachment[]`.

### GET /api/attachments/{hashedName}
- 파일 스트림. `Content-Disposition: attachment; filename*=UTF-8''<originalName>` 로 원본 이름 복원.
- `<a href={attachment.downloadUrl} download>` 로 바로 사용 가능.

### DELETE /api/attachments/{hashedName}
- 게시물 작성자만. DB 행과 디스크 파일 삭제. 204.

## 본문 이미지

### POST /api/images
`multipart/form-data`, 로그인 필요. 게시물 본문(마크다운)에 삽입할 이미지를 업로드한다.
릴리즈 첨부파일과 달리 게시물이 저장되기 전(작성 중)에도 올릴 수 있어 `postId` 를 받지 않는다.

| 필드 | 값 |
|---|---|
| `file` | 이미지 파일 (filename 필수, png/jpg/jpeg/gif/webp 만 허용) |

```ts
const form = new FormData()
form.append('file', file)
const { url } = await fetch('/api/images', { method: 'POST', body: form }).then((r) => r.json())
// url 을 그대로 마크다운에 삽입: ![alt](${url})
```
- 응답 `url` 은 **상대 경로** (`/api/images/{storedName}`). 절대 URL로 고정해서 저장하지 않는 이유:
  도메인이 나중에 바뀌면(IP → 커스텀 도메인 등) 이미 저장된 게시물에 박힌 절대 URL이 깨지기 때문.
  마크다운에는 이 상대 경로를 그대로 삽입한다 (base64 데이터 URI로 넣으면 Discord 썸네일이 표시되지 않는다).
  Discord 웹훅처럼 외부에서 접근 가능한 절대 URL이 필요한 곳(썸네일)은, 알림을 보내는 시점에 그때그때
  현재 `APP_BASE_URL` 로 변환한다 (`services/discord.rs::resolve_image_url`) — 업로드 시점이 아니라
  전송 시점 기준이라 도메인이 바뀐 뒤에도 예전 게시물의 썸네일이 계속 정상 동작한다.
- 201 + `{ url }`. 지원하지 않는 확장자 400. 크기 초과 413.

### GET /api/images/{storedName}
- 공개, 인증 불필요. 이미지 스트림 (`Cache-Control: public, max-age=31536000, immutable`).

## 태그

### GET /api/tags
- 200 + `TagCount[]`. 게시물이 1개 이상인 태그만, 많이 쓰인 순.

## 프론트엔드 연동 현황

`src/api/` 의 fetch 래퍼를 통해 아래 화면이 연동되어 있다.

| 화면 | 사용하는 API |
|---|---|
| Layout / Header | `GET /api/auth/me` (세션 확인, 닉네임 표시) |
| LoginPage | `POST /api/auth/register`, `POST /api/auth/login`, `GET /api/auth/providers`, GitHub 버튼 → `/api/auth/github` (비밀번호 확인·규칙은 클라이언트에서 선검사) |
| HomePage | `GET /api/posts` (검색창 → `tag`/`user`/`q`, 250ms 디바운스) |
| PostDetailPage | `GET /api/posts/{id}`, 댓글 목록/작성/삭제, 별 토글, 게시물 삭제, 첨부 다운로드 |
| WritePage | `POST /api/posts` → 첨부파일마다 `POST /api/posts/{id}/attachments`, 임시저장은 `GET`/`PUT`/`DELETE /api/users/me/draft` |
| MyPage | `GET /api/users/me/posts`, `PATCH /api/users/me`, `POST /api/auth/logout` |

아직 UI 가 없는 API: 댓글 수정, 게시물 수정, 비밀번호 변경, 다른 사용자 프로필/게시물, 태그 목록.
