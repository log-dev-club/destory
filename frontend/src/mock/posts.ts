import type { Post } from '../types'

export const mockPosts: Post[] = [
  {
    id: '1',
    title: 'React 19 useTransition으로 검색 UX 개선하기',
    excerpt: '입력 지연 없이 대량 리스트를 필터링하는 방법을 정리했습니다.',
    content: `대량의 리스트를 실시간으로 필터링하다 보면 입력할 때마다 화면이 버벅이는 문제가 생깁니다.
이번 글에서는 \`useTransition\`으로 급하지 않은 렌더링을 뒤로 미루는 방법을 정리합니다.

## 문제 상황

- 입력 이벤트마다 수천 건의 리스트를 필터링
- 타이핑 중 프레임 드랍 발생

## 해결 코드

\`\`\`tsx
import { useTransition, useState } from 'react'

function SearchableList({ items }: { items: Item[] }) {
  const [query, setQuery] = useState('')
  const [isPending, startTransition] = useTransition()
  const [filtered, setFiltered] = useState(items)

  const handleChange = (value: string) => {
    setQuery(value)
    startTransition(() => {
      setFiltered(items.filter((item) => item.name.includes(value)))
    })
  }

  return (
    <>
      <input value={query} onChange={(e) => handleChange(e.target.value)} />
      {isPending && <span>검색 중...</span>}
      <List items={filtered} />
    </>
  )
}
\`\`\`

> 입력 자체는 즉시 반영하고, 무거운 필터링만 \`startTransition\`으로 감싸는 것이 핵심입니다.

이렇게 하면 입력 지연 없이도 부드러운 검색 UX를 만들 수 있습니다.`,
    author: { nickname: 'yangmin' },
    tags: ['react', 'frontend'],
    createdAt: '2026-09-01',
    commentCount: 4,
    starCount: 12,
  },
  {
    id: '2',
    title: 'Axum + SQLx로 MySQL 커넥션 풀 구성하기',
    excerpt: '동아리 프로젝트 백엔드 초기 세팅 과정을 기록합니다.',
    content: `axum과 sqlx로 백엔드 초기 세팅을 하면서 커넥션 풀을 구성한 과정을 정리합니다.

## 의존성

\`\`\`toml
[dependencies]
axum = "0.8"
tokio = { version = "1.40", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "mysql"] }
\`\`\`

## 풀 생성

\`\`\`rust
use sqlx::mysql::MySqlPoolOptions;

let pool = MySqlPoolOptions::new()
    .max_connections(10)
    .connect(&database_url)
    .await?;
\`\`\`

풀 크기는 워커 수와 DB 최대 커넥션 한도를 함께 고려해서 정했습니다.`,
    author: { nickname: 'jaehun' },
    tags: ['rust', 'backend', 'axum'],
    createdAt: '2026-08-28',
    commentCount: 2,
    starCount: 7,
  },
  {
    id: '3',
    title: '깃허브 테마를 재현하는 CSS 변수 설계',
    excerpt: '라이트/다크 전환을 대비한 디자인 토큰 구조를 소개합니다.',
    content: `색상을 하드코딩하지 않고 CSS 변수(디자인 토큰)로 뽑아두면 테마 전환이 쉬워집니다.

\`\`\`css
:root {
  --color-canvas-default: #0d1117;
  --color-fg-default: #e6edf3;
  --color-accent-fg: #4493f8;
}

body {
  background-color: var(--color-canvas-default);
  color: var(--color-fg-default);
}
\`\`\`

컴포넌트 CSS에서는 항상 변수만 참조하도록 규칙을 정해두면, 루트 변수만 바꿔서 다크 테마 전체를 적용할 수 있습니다.`,
    author: { nickname: 'yangmin' },
    tags: ['css', 'design-system'],
    createdAt: '2026-08-20',
    commentCount: 1,
    starCount: 5,
  },
  {
    id: '4',
    title: '릴리즈 파일 해싱 업로드 흐름 설계기',
    excerpt: '전송 시간과 파일명을 조합해 파일명을 해싱하는 방식을 공유합니다.',
    content: `릴리즈 파일(zip, exe 등)을 업로드할 때 원본 파일명을 그대로 저장하면 충돌·추측 문제가 생길 수 있습니다.
그래서 전송 시각과 파일명을 조합해 해싱한 이름으로 저장하는 방식을 적용했습니다.

\`\`\`ts
async function hashReleaseFileName(file: File, sentAt: number) {
  const source = \`\${sentAt}:\${file.name}\`
  const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(source))
  return Array.from(new Uint8Array(digest))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')
}
\`\`\`

서버/DB는 해싱된 이름만 참조하고, 사용자에게는 원본 파일명을 그대로 보여주는 매핑 테이블을 별도로 둡니다.`,
    author: { nickname: 'dohyun' },
    tags: ['upload', 'security'],
    createdAt: '2026-08-15',
    commentCount: 6,
    starCount: 20,
  },
  {
    id: '5',
    title: 'Vite 환경 변수와 배포 환경 분리하기',
    excerpt: '개발, 스테이징, 프로덕션 환경에서 설정값을 안전하게 관리하는 방법을 정리했습니다.',
    content: `Vite는 \`.env.[mode]\` 파일로 환경별 설정을 분리할 수 있습니다.

\`\`\`bash
.env.development
.env.staging
.env.production
\`\`\`

\`\`\`ts
const apiBaseUrl = import.meta.env.VITE_API_BASE_URL
\`\`\`

클라이언트에 노출되면 안 되는 값은 \`VITE_\` 접두사를 붙이지 않아 번들에 포함되지 않도록 주의했습니다.`,
    author: { nickname: 'seoyeon' },
    tags: ['vite', 'deployment'],
    createdAt: '2026-08-10',
    commentCount: 3,
    starCount: 9,
  },
  {
    id: '6',
    title: 'TypeScript 타입 가드로 API 응답 다루기',
    excerpt: '외부 API의 불확실한 응답을 런타임에서 안전하게 검증하는 패턴을 살펴봅니다.',
    content: `외부 API 응답은 타입 시스템만으로 보장되지 않기 때문에 런타임 검증이 필요합니다.

\`\`\`ts
interface UserResponse {
  id: string
  nickname: string
}

function isUserResponse(value: unknown): value is UserResponse {
  return (
    typeof value === 'object' &&
    value !== null &&
    typeof (value as UserResponse).id === 'string' &&
    typeof (value as UserResponse).nickname === 'string'
  )
}
\`\`\`

이렇게 타입 가드를 만들어두면 \`fetch\` 응답을 안전하게 좁혀서 사용할 수 있습니다.`,
    author: { nickname: 'minsu' },
    tags: ['typescript', 'api'],
    createdAt: '2026-08-06',
    commentCount: 5,
    starCount: 14,
  },
  {
    id: '7',
    title: 'PostgreSQL 인덱스로 목록 조회 최적화하기',
    excerpt: '자주 사용하는 정렬과 필터 조건에 맞춰 인덱스를 설계한 경험을 기록합니다.',
    content: `목록 조회 쿼리가 느려져서 실행 계획을 분석하고 인덱스를 추가했습니다.

\`\`\`sql
CREATE INDEX idx_posts_created_at_desc
ON posts (created_at DESC);

EXPLAIN ANALYZE
SELECT * FROM posts
ORDER BY created_at DESC
LIMIT 20;
\`\`\`

정렬 컬럼과 자주 쓰는 필터 컬럼을 복합 인덱스로 묶으니 조회 시간이 크게 줄었습니다.`,
    author: { nickname: 'jiwon' },
    tags: ['database', 'postgresql'],
    createdAt: '2026-07-30',
    commentCount: 2,
    starCount: 11,
  },
  {
    id: '8',
    title: 'React 컴포넌트 테스트를 작게 시작하는 법',
    excerpt: '사용자 행동 중심의 테스트 케이스를 만들며 컴포넌트 신뢰도를 높여봅니다.',
    content: `구현 세부사항이 아니라 사용자 행동을 기준으로 테스트를 작성하면 리팩터링에 강해집니다.

\`\`\`tsx
import { render, screen, fireEvent } from '@testing-library/react'

test('버튼을 누르면 카운트가 증가한다', () => {
  render(<Counter />)
  fireEvent.click(screen.getByRole('button', { name: '증가' }))
  expect(screen.getByText('1')).toBeInTheDocument()
})
\`\`\`

작은 테스트부터 쌓아가는 것이 커버리지를 억지로 채우는 것보다 훨씬 오래 유지보수됩니다.`,
    author: { nickname: 'hyejin' },
    tags: ['react', 'testing'],
    createdAt: '2026-07-24',
    commentCount: 4,
    starCount: 16,
  },
  {
    id: '9',
    title: 'Rust 에러 타입을 읽기 쉽게 설계하기',
    excerpt: '서비스 계층에서 발생하는 오류를 호출자에게 명확하게 전달하는 구조를 소개합니다.',
    content: `\`thiserror\`로 도메인 에러를 명확하게 정의하면 호출자가 오류를 다루기 쉬워집니다.

\`\`\`rust
#[derive(thiserror::Error, Debug)]
enum PostError {
    #[error("게시물을 찾을 수 없습니다: {0}")]
    NotFound(String),
    #[error("데이터베이스 오류")]
    Database(#[from] sqlx::Error),
}
\`\`\`

에러 메시지에 원인을 함께 담아두면 로그만 보고도 문제를 빠르게 좁힐 수 있습니다.`,
    author: { nickname: 'junho' },
    tags: ['rust', 'error-handling'],
    createdAt: '2026-07-18',
    commentCount: 7,
    starCount: 18,
  },
  {
    id: '10',
    title: '웹 접근성을 위한 폼 에러 메시지 개선',
    excerpt: '키보드 사용자와 스크린 리더 사용자가 오류를 놓치지 않도록 폼을 개선해봅니다.',
    content: `오류 메시지를 시각적으로만 표시하면 스크린 리더 사용자는 알아채기 어렵습니다.

\`\`\`html
<label for="email">이메일</label>
<input id="email" aria-describedby="email-error" aria-invalid="true" />
<p id="email-error" role="alert">올바른 이메일 형식이 아닙니다.</p>
\`\`\`

\`aria-describedby\`로 입력과 에러 메시지를 연결하고, \`role="alert"\`로 즉시 안내되도록 했습니다.`,
    author: { nickname: 'eunchae' },
    tags: ['accessibility', 'html'],
    createdAt: '2026-07-12',
    commentCount: 1,
    starCount: 8,
  },
]
