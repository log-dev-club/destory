// 백엔드 DTO 와 1:1 (docs/api.md 참고). 필드명은 camelCase.

export interface Author {
  nickname: string
  avatarUrl?: string
}

/** 목록 항목 (GET /api/posts). content 없음 */
export interface PostSummary {
  id: number
  title: string
  excerpt: string
  author: Author
  tags: string[]
  createdAt: string // RFC 3339
  updatedAt: string
  commentCount: number
  starCount: number
  starred: boolean
  hasImage: boolean
}

/** 상세 (GET /api/posts/{id}) */
export interface PostDetail extends PostSummary {
  content: string
  attachments: Attachment[]
}

export interface PostPage {
  items: PostSummary[]
  page: number
  limit: number
  total: number
}

export interface Comment {
  id: number
  postId: number
  content: string
  author: Author
  createdAt: string
  updatedAt: string
}

export interface Attachment {
  id: number
  postId: number
  originalName: string
  hashedName: string
  size: number
  sentAt: number // ms epoch
  downloadUrl: string
  createdAt: string
}

/** POST /api/images 응답. url 을 그대로 마크다운 이미지 문법에 삽입 */
export interface UploadedImage {
  url: string
}

export interface UserProfile {
  id: number
  nickname: string
  avatarUrl?: string
  bio?: string
  /** GitHub 로 가입/연동한 경우 GitHub 로그인명 */
  githubLogin?: string
  createdAt: string
  postCount: number
  starCount: number
}

export interface TagCount {
  name: string
  postCount: number
}
