import { useEffect, useState } from 'react'
import { Link, useOutletContext } from 'react-router-dom'
import PostCard from '../components/PostCard'
import type { LayoutContext } from '../components/Layout'
import { fetchPosts } from '../api/posts'
import { isEmptyQuery, parseSearchQuery, toPostListParams } from '../utils/searchQuery'
import type { PostPage } from '../types'
import './HomePage.css'

const SEARCH_DEBOUNCE_MS = 250

function HomePage() {
  const { query, user } = useOutletContext<LayoutContext>()
  const [page, setPage] = useState<PostPage | null>(null)
  const [error, setError] = useState<string | null>(null)

  // 검색어가 바뀔 때마다 서버에서 다시 조회 (타이핑 중 과다 요청 방지용 디바운스)
  useEffect(() => {
    const parsed = parseSearchQuery(query)
    const timer = setTimeout(() => {
      setError(null)
      fetchPosts(toPostListParams(parsed))
        .then(setPage)
        .catch((err: Error) => setError(err.message))
    }, isEmptyQuery(parsed) ? 0 : SEARCH_DEBOUNCE_MS)
    return () => clearTimeout(timer)
  }, [query])

  return (
    <div className="home-page">
      <div className="home-page__header">
        <h1></h1>
        <Link className="home-page__write-link" to={user ? '/write' : '/login'} state={{ from: '/write' }}>
          Add
        </Link>
      </div>
      {error ? (
        <p className="home-page__empty">{error}</p>
      ) : page === null ? (
        <p className="home-page__empty">불러오는 중...</p>
      ) : page.items.length === 0 ? (
        <p className="home-page__empty">
          {isEmptyQuery(parseSearchQuery(query)) ? '아직 게시물이 없습니다.' : '검색 결과가 없습니다.'}
        </p>
      ) : (
        <div className="home-page__list">
          {page.items.map((post) => (
            <PostCard key={post.id} post={post} />
          ))}
        </div>
      )}
    </div>
  )
}

export default HomePage
