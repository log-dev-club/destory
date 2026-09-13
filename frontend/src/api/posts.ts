import { api, toQuery } from './client'
import type { PostDetail, PostPage } from '../types'

export interface PostListParams {
  tag?: string // 쉼표 구분
  user?: string
  q?: string // 공백 구분
  page?: number
  limit?: number
}

export const fetchPosts = (params: PostListParams = {}) =>
  api<PostPage>(`/api/posts${toQuery(params)}`)

export const fetchMyPosts = (params: PostListParams = {}) =>
  api<PostPage>(`/api/users/me/posts${toQuery(params)}`)

export const fetchPost = (id: number) => api<PostDetail>(`/api/posts/${id}`)

export interface CreatePostBody {
  title: string
  content: string
  excerpt?: string
  tags?: string[]
}

export const createPost = (body: CreatePostBody) =>
  api<PostDetail>('/api/posts', { method: 'POST', body: JSON.stringify(body) })

export const updatePost = (id: number, body: Partial<CreatePostBody>) =>
  api<PostDetail>(`/api/posts/${id}`, { method: 'PATCH', body: JSON.stringify(body) })

export const deletePost = (id: number) => api<void>(`/api/posts/${id}`, { method: 'DELETE' })

export interface StarResponse {
  starred: boolean
  starCount: number
}

export const starPost = (id: number) => api<StarResponse>(`/api/posts/${id}/star`, { method: 'PUT' })

export const unstarPost = (id: number) =>
  api<StarResponse>(`/api/posts/${id}/star`, { method: 'DELETE' })
