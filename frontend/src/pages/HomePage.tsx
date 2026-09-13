import { useMemo } from 'react'
import { useOutletContext } from 'react-router-dom'
import PostCard from '../components/PostCard'
import { mockPosts } from '../mock/posts'
import { matchesSearchQuery, parseSearchQuery } from '../utils/searchQuery'
import './HomePage.css'

interface OutletContext {
  query: string
}

function HomePage() {
  const { query } = useOutletContext<OutletContext>()

  const filteredPosts = useMemo(() => {
    const parsed = parseSearchQuery(query)
    if (parsed.tags.length === 0 && parsed.users.length === 0 && parsed.text.length === 0) {
      return mockPosts
    }
    return mockPosts.filter((post) => matchesSearchQuery(post, parsed))
  }, [query])

  return (
    <div className="home-page">
      <div className="home-page__header">
        <h1>최신 게시물</h1>
        <a className="home-page__write-link" href="/write">
          Add
        </a>
      </div>
      {filteredPosts.length === 0 ? (
        <p className="home-page__empty">검색 결과가 없습니다.</p>
      ) : (
        <div className="home-page__list">
          {filteredPosts.map((post) => (
            <PostCard key={post.id} post={post} />
          ))}
        </div>
      )}
    </div>
  )
}

export default HomePage
