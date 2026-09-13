import { useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import CommentSection from '../components/CommentSection'
import Markdown from '../components/Markdown'
import { CURRENT_USER_NICKNAME } from '../constants'
import { mockPosts } from '../mock/posts'
import type { Comment } from '../types'
import './PostDetailPage.css'

function PostDetailPage() {
  const { id } = useParams<{ id: string }>()
  const post = mockPosts.find((item) => item.id === id)
  const [comments, setComments] = useState<Comment[]>(post?.comments ?? [])

  if (!post) {
    return (
      <div className="post-detail post-detail--empty">
        <p>게시물을 찾을 수 없습니다.</p>
        <Link to="/">목록으로 돌아가기</Link>
      </div>
    )
  }

  const handleAddComment = (content: string) => {
    const newComment: Comment = {
      id: `${post.id}-${Date.now()}`,
      author: { nickname: CURRENT_USER_NICKNAME },
      content,
      createdAt: new Date().toISOString().slice(0, 10),
    }
    setComments((prev) => [...prev, newComment])
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
          <span>💬 {comments.length}</span>
        </div>
      </div>

      <div className="post-detail__body">
        <Markdown content={post.content} />
      </div>

      <CommentSection comments={comments} onAddComment={handleAddComment} />
    </article>
  )
}

export default PostDetailPage
