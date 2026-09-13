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

export interface AuthProviders {
  github: boolean
}

/** 서버에 설정된 외부 로그인 제공자. 버튼 표시 여부 판단용 */
export const fetchProviders = () => api<AuthProviders>('/api/auth/providers')

/** GitHub OAuth 시작. 브라우저 전체가 GitHub 로 이동했다가 콜백 후 돌아온다 */
export function startGithubLogin() {
  window.location.href = '/api/auth/github'
}

/** 비로그인이면 null */
export async function fetchMe(): Promise<UserProfile | null> {
  try {
    return await api<UserProfile>('/api/auth/me')
  } catch (error) {
    if (error instanceof ApiError && error.status === 401) return null
    throw error
  }
}
