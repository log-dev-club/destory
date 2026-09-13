import { Link, useParams } from 'react-router-dom'
import Markdown from '../components/Markdown'
import { mockPosts } from '../mock/posts'
import './PostDetailPage.css'

function PostDetailPage() {
  const { id } = useParams<{ id: string }>()
  const post = mockPosts.find((item) => item.id === id)

  if (!post) {
    return (
      <div className="post-detail post-detail--empty">
        <p>게시물을 찾을 수 없습니다.</p>
        <Link to="/">목록으로 돌아가기</Link>
      </div>
    )
  }

  return (
    <article className="post-detail">
      <Link className="post-detail__back" to="/">
        ← 목록으로
      </Link>

      <h1 className="post-detail__title">{post.title}</h1>

      <div className="post-detail__meta">
        <span className="post-detail__author">{post.author.nickname}</span>
        <span className="post-detail__dot">·</span>
        <time>{post.createdAt}</time>
        <div className="post-detail__tags">
          {post.tags.map((tag) => (
            <span key={tag} className="post-detail__tag">
              #{tag}
            </span>
          ))}
        </div>
        <div className="post-detail__stats">
          <span>★ {post.starCount}</span>
          <span>💬 {post.commentCount}</span>
        </div>
      </div>

      <div className="post-detail__body">
        <Markdown content={post.content} />
      </div>
    </article>
  )
}

export default PostDetailPage
