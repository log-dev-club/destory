import { useCallback, useEffect, useState } from 'react'
import { Outlet } from 'react-router-dom'
import Header from './Header'
import { fetchMe } from '../api/auth'
import type { UserProfile } from '../types'

/** 모든 페이지가 useOutletContext<LayoutContext>() 로 받는 값 */
export interface LayoutContext {
  query: string
  /** 로그인 사용자. 비로그인이면 null, 아직 확인 전이면 undefined */
  user: UserProfile | null | undefined
  setUser: (user: UserProfile | null) => void
  refreshUser: () => Promise<void>
}

function Layout() {
  const [query, setQuery] = useState('')
  const [user, setUser] = useState<UserProfile | null | undefined>(undefined)

  const refreshUser = useCallback(async () => {
    try {
      setUser(await fetchMe())
    } catch {
      setUser(null)
    }
  }, [])

  // 최초 진입 시 세션 쿠키로 로그인 상태 확인
  useEffect(() => {
    let cancelled = false
    fetchMe()
      .then((me) => {
        if (!cancelled) setUser(me)
      })
      .catch(() => {
        if (!cancelled) setUser(null)
      })
    return () => {
      cancelled = true
    }
  }, [])

  return (
    <div className="app-shell">
      <Header query={query} onQueryChange={setQuery} user={user ?? null} />
      <main className="app-content">
        <Outlet context={{ query, user, setUser, refreshUser } satisfies LayoutContext} />
      </main>
    </div>
  )
}

export default Layout
