import { mockPosts } from '../mock/posts'
import PostCard from '../components/PostCard'
import { CURRENT_USER_NICKNAME } from '../constants'
import './MyPage.css'

function MyPage() {
  const myPosts = mockPosts.filter((post) => post.author.nickname === CURRENT_USER_NICKNAME)

  return (
    <div className="my-page">
      <div className="my-page__profile">
        <div className="my-page__avatar">
          <svg viewBox="0 0 24 24" width="48" height="48" aria-hidden="true">
            <circle cx="12" cy="8" r="4" fill="currentColor" />
            <path fill="currentColor" d="M4 20c0-4.4 3.6-8 8-8s8 3.6 8 8v1H4z" />
          </svg>
        </div>
        <div>
          <h1>{CURRENT_USER_NICKNAME}</h1>
          <p className="my-page__stats">
            게시물 {myPosts.length}개 · 받은 별 {myPosts.reduce((sum, post) => sum + post.starCount, 0)}개
          </p>
        </div>
      </div>

      <h2 className="my-page__section-title">내가 작성한 게시물</h2>
      {myPosts.length === 0 ? (
        <p className="my-page__empty">아직 작성한 게시물이 없습니다.</p>
      ) : (
        <div className="my-page__list">
          {myPosts.map((post) => (
            <PostCard key={post.id} post={post} />
          ))}
        </div>
      )}
    </div>
  )
}

export default MyPage
