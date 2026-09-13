import { useState } from 'react'
import type { FormEvent } from 'react'
import type { Comment } from '../types'
import { CURRENT_USER_NICKNAME } from '../constants'
import './CommentSection.css'

interface CommentSectionProps {
  comments: Comment[]
  onAddComment: (content: string) => void
}

function CommentSection({ comments, onAddComment }: CommentSectionProps) {
  const [content, setContent] = useState('')

  const handleSubmit = (event: FormEvent) => {
    event.preventDefault()
    const trimmed = content.trim()
    if (!trimmed) return

    onAddComment(trimmed)
    setContent('')
  }

  return (
    <section className="comment-section">
      <h2 className="comment-section__title">댓글 {comments.length}개</h2>

      {comments.length === 0 ? (
        <p className="comment-section__empty">아직 댓글이 없습니다. 첫 댓글을 남겨보세요.</p>
      ) : (
        <ul className="comment-list">
          {comments.map((comment) => (
            <li key={comment.id} className="comment-list__item">
              <div className="comment-list__meta">
                <span className="comment-list__author">{comment.author.nickname}</span>
                <span className="comment-list__dot">·</span>
                <time>{comment.createdAt}</time>
              </div>
              <p className="comment-list__content">{comment.content}</p>
            </li>
          ))}
        </ul>
      )}

      <form className="comment-form" onSubmit={handleSubmit}>
        <label className="comment-form__field">
          <span>{CURRENT_USER_NICKNAME}(으)로 댓글 작성</span>
          <textarea
            value={content}
            onChange={(event) => setContent(event.target.value)}
            rows={3}
            placeholder="댓글을 입력하세요"
          />
        </label>
        <button type="submit" className="comment-form__submit" disabled={!content.trim()}>
          댓글 작성
        </button>
      </form>
    </section>
  )
}

export default CommentSection
