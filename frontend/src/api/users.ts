import { api } from './client'
import type { UserProfile } from '../types'

export interface UpdateProfileBody {
  nickname?: string
  avatarUrl?: string // '' 이면 삭제
  bio?: string // '' 이면 삭제
}

export const fetchMyProfile = () => api<UserProfile>('/api/users/me')

export const updateMyProfile = (body: UpdateProfileBody) =>
  api<UserProfile>('/api/users/me', { method: 'PATCH', body: JSON.stringify(body) })

export const changePassword = (currentPassword: string, newPassword: string) =>
  api<void>('/api/users/me/password', {
    method: 'PUT',
    body: JSON.stringify({ currentPassword, newPassword }),
  })

export const fetchUserProfile = (nickname: string) =>
  api<UserProfile>(`/api/users/${encodeURIComponent(nickname)}`)
