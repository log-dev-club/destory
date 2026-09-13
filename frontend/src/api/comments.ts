import { api } from './client'
import type { Comment } from '../types'

export const fetchComments = (postId: number) => api<Comment[]>(`/api/posts/${postId}/comments`)

export const createComment = (postId: number, content: string) =>
  api<Comment>(`/api/posts/${postId}/comments`, {
    method: 'POST',
    body: JSON.stringify({ content }),
  })

export const updateComment = (id: number, content: string) =>
  api<Comment>(`/api/comments/${id}`, { method: 'PATCH', body: JSON.stringify({ content }) })

export const deleteComment = (id: number) => api<void>(`/api/comments/${id}`, { method: 'DELETE' })
