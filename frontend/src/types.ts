export interface Author {
  nickname: string
  avatarUrl?: string
}

export interface Post {
  id: string
  title: string
  excerpt: string
  content: string
  author: Author
  tags: string[]
  createdAt: string
  commentCount: number
  starCount: number
}
