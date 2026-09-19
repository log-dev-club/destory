import { useEffect, useState } from 'react'
import type { FormEvent } from 'react'
import { Link, useNavigate, useOutletContext } from 'react-router-dom'
import type { LayoutContext } from '../components/Layout'
import { fetchAdminUsers, setUserRole } from '../api/admin'
import { formatDate } from '../utils/formatDate'
import type { AdminUserPage, AdminUserSummary } from '../types'
import './AdminPage.css'

const ROLE_LABEL: Record<string, string> = {
  user: '일반',
  admin: '관리자',
  super_admin: '최고 관리자',
}

const LIMIT = 20

function AdminPage() {
  const { user: viewer } = useOutletContext<LayoutContext>()
  const navigate = useNavigate()

  const [q, setQ] = useState('')
  const [page, setPage] = useState(1)
  const [result, setResult] = useState<AdminUserPage | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [pendingNickname, setPendingNickname] = useState<string | null>(null)

  const isAdmin = viewer && (viewer.role === 'admin' || viewer.role === 'super_admin')

  // 관리자가 아니면 접근 불가
  useEffect(() => {
    if (viewer === null || (viewer && !isAdmin)) navigate('/', { replace: true })
  }, [viewer, isAdmin, navigate])

  const load = (targetPage: number, query: string) => {
    setError(null)
    fetchAdminUsers({ q: query, page: targetPage, limit: LIMIT })
      .then(setResult)
      .catch((err: Error) => setError(err.message))
  }

  useEffect(() => {
    if (!isAdmin) return
    load(page, q)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isAdmin, page])

  const submitSearch = (event: FormEvent) => {
    event.preventDefault()
    setPage(1)
    load(1, q)
  }

  const toggleAdminRole = async (target: AdminUserSummary) => {
    setPendingNickname(target.nickname)
    setError(null)
    try {
      const nextRole = target.role === 'admin' ? 'user' : 'admin'
      await setUserRole(target.nickname, nextRole)
      load(page, q)
    } catch (err) {
      setError(err instanceof Error ? err.message : '역할 변경에 실패했습니다')
    } finally {
      setPendingNickname(null)
    }
  }

  if (!isAdmin) {
    return (
      <div className="admin-page">
        <p className="admin-page__empty">불러오는 중...</p>
      </div>
    )
  }

  const totalPages = result ? Math.max(1, Math.ceil(result.total / LIMIT)) : 1

  return (
    <div className="admin-page">
      <h1 className="admin-page__title">사용자 관리</h1>

      <form className="admin-page__search" onSubmit={submitSearch}>
        <input
          type="text"
          value={q}
          onChange={(e) => setQ(e.target.value)}
          placeholder="닉네임 검색"
        />
        <button type="submit">검색</button>
      </form>

      {error && <p className="admin-page__error">{error}</p>}

      {result === null ? (
        <p className="admin-page__empty">불러오는 중...</p>
      ) : result.items.length === 0 ? (
        <p className="admin-page__empty">사용자가 없습니다.</p>
      ) : (
        <table className="admin-page__table">
          <thead>
            <tr>
              <th>닉네임</th>
              <th>역할</th>
              <th>게시물</th>
              <th>가입일</th>
              {viewer.role === 'super_admin' && <th>관리</th>}
            </tr>
          </thead>
          <tbody>
            {result.items.map((item) => (
              <tr key={item.id}>
                <td>
                  <Link to={`/users/${encodeURIComponent(item.nickname)}`}>{item.nickname}</Link>
                </td>
                <td>
                  <span className={`admin-page__role admin-page__role--${item.role}`}>
                    {ROLE_LABEL[item.role]}
                  </span>
                </td>
                <td>{item.postCount}</td>
                <td>{formatDate(item.createdAt)}</td>
                {viewer.role === 'super_admin' && (
                  <td>
                    {item.role !== 'super_admin' && item.nickname !== viewer.nickname && (
                      <button
                        type="button"
                        onClick={() => toggleAdminRole(item)}
                        disabled={pendingNickname === item.nickname}
                      >
                        {item.role === 'admin' ? '관리자 해제' : '관리자 지정'}
                      </button>
                    )}
                  </td>
                )}
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {result && totalPages > 1 && (
        <div className="admin-page__pagination">
          <button type="button" disabled={page <= 1} onClick={() => setPage((p) => p - 1)}>
            이전
          </button>
          <span>
            {page} / {totalPages}
          </span>
          <button type="button" disabled={page >= totalPages} onClick={() => setPage((p) => p + 1)}>
            다음
          </button>
        </div>
      )}
    </div>
  )
}

export default AdminPage
