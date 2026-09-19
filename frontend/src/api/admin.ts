import { api, toQuery } from './client'
import type { AdminUserPage, UserProfile, UserRole } from '../types'

export interface AdminUserListParams {
  q?: string
  page?: number
  limit?: number
}

export const fetchAdminUsers = (params: AdminUserListParams = {}) =>
  api<AdminUserPage>(`/api/admin/users${toQuery(params)}`)

/** 최고 관리자만 호출 가능. role 은 'user' | 'admin' */
export const setUserRole = (nickname: string, role: Extract<UserRole, 'user' | 'admin'>) =>
  api<UserProfile>(`/api/admin/users/${encodeURIComponent(nickname)}/role`, {
    method: 'PUT',
    body: JSON.stringify({ role }),
  })
