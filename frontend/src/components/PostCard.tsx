import { Link } from 'react-router-dom'
import type { PostSummary } from '../types'
import { formatDate } from '../utils/formatDate'
import './PostCard.css'

interface PostCardProps {
  post: PostSummary
}

function PostCard({ post }: PostCardProps) {
  return (
    <article className="post-card">
      <h3 className="post-card__title">
        <Link to={`/posts/${post.id}`}>{post.title}</Link>
      </h3>
      <p className="post-card__excerpt">{post.excerpt}</p>
      <div className="post-card__meta">
        <span className="post-card__author">{post.author.nickname}</span>
        <span className="post-card__dot">·</span>
        <time className="post-card__date" dateTime={post.createdAt}>
          {formatDate(post.createdAt)}
        </time>
        <div className="post-card__tags">
          {post.tags.map((tag) => (
            <span key={tag} className="post-card__tag">
              #{tag}
            </span>
          ))}
        </div>
        <div className="post-card__stats">
          <span>{post.starred ? '★' : '☆'} {post.starCount}</span>
          <span>💬 {post.commentCount}</span>
        </div>
      </div>
    </article>
  )
}

export default PostCard
