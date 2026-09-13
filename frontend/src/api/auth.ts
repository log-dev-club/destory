import { api, ApiError } from './client'
import type { UserProfile } from '../types'

export interface Credentials {
  nickname: string
  password: string
}

export const register = (body: Credentials) =>
  api<UserProfile>('/api/auth/register', { method: 'POST', body: JSON.stringify(body) })

export const login = (body: Credentials) =>
  api<UserProfile>('/api/auth/login', { method: 'POST', body: JSON.stringify(body) })

export const logout = () => api<void>('/api/auth/logout', { method: 'POST' })

/** 비로그인이면 null */
export async function fetchMe(): Promise<UserProfile | null> {
  try {
    return await api<UserProfile>('/api/auth/me')
  } catch (error) {
    if (error instanceof ApiError && error.status === 401) return null
    throw error
  }
}
