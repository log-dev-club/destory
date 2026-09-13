/**
 * 공통 fetch 래퍼. 상대 경로 /api/... 로 호출하면 Vite 프록시가 백엔드로 전달한다.
 * 실패하면 서버의 { error } 메시지를 담은 ApiError 를 던진다.
 */
export class ApiError extends Error {
  status: number

  constructor(status: number, message: string) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

function extractError(data: unknown, status: number): string {
  if (typeof data === 'object' && data !== null && 'error' in data) {
    const message = (data as { error: unknown }).error
    if (typeof message === 'string') return message
  }
  return `요청에 실패했습니다 (${status})`
}

export async function api<T>(path: string, init: RequestInit = {}): Promise<T> {
  const headers = new Headers(init.headers)
  if (init.body !== undefined && !(init.body instanceof FormData)) {
    headers.set('Content-Type', 'application/json')
  }

  const response = await fetch(path, { ...init, headers })
  if (response.status === 204) return undefined as T

  const data: unknown = await response.json().catch(() => null)
  if (!response.ok) {
    throw new ApiError(response.status, extractError(data, response.status))
  }
  return data as T
}

/** 객체를 쿼리 문자열로. undefined/빈 문자열은 제외 */
export function toQuery<T extends object>(params: T): string {
  const search = new URLSearchParams()
  for (const [key, value] of Object.entries(params) as [string, unknown][]) {
    if (typeof value === 'number' || (typeof value === 'string' && value !== '')) {
      search.set(key, String(value))
    }
  }
  const text = search.toString()
  return text ? `?${text}` : ''
}
