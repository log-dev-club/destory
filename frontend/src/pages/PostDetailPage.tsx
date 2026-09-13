import { useEffect, useState } from 'react'
import type { FormEvent } from 'react'
import { Link, useNavigate, useOutletContext, useParams } from 'react-router-dom'
import Markdown from '../components/Markdown'
import type { LayoutContext } from '../components/Layout'
import { deletePost, fetchPost, starPost, unstarPost } from '../api/posts'
import { createComment, deleteComment, fetchComments } from '../api/comments'
import { formatBytes, formatDate, formatDateTime } from '../utils/formatDate'
import type { Comment, PostDetail } from '../types'
import './PostDetailPage.css'

function PostDetailPage() {
  const { id } = useParams<{ id: string }>()
  const postId = Number(id)
  const { user } = useOutletContext<LayoutContext>()
  const navigate = useNavigate()

  const invalidId = !Number.isInteger(postId)

  const [post, setPost] = useState<PostDetail | null>(null)
  const [comments, setComments] = useState<Comment[]>([])
  const [loadFailed, setLoadFailed] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [commentText, setCommentText] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const notFound = invalidId || loadFailed

  useEffect(() => {
    if (invalidId) return
    let cancelled = false
    Promise.all([fetchPost(postId), fetchComments(postId)])
      .then(([loadedPost, loadedComments]) => {
        if (cancelled) return
        setPost(loadedPost)
        setComments(loadedComments)
      })
      .catch(() => {
        if (!cancelled) setLoadFailed(true)
      })
    return () => {
      cancelled = true
    }
  }, [postId, invalidId])

  const isOwner = user !== null && user !== undefined && post !== null && user.nickname === post.author.nickname

  const toggleStar = async () => {
    if (!post) return
    if (!user) {
      navigate('/login', { state: { from: `/posts/${post.id}` } })
      return
    }
    try {
      const result = post.starred ? await unstarPost(post.id) : await starPost(post.id)
      setPost({ ...post, starred: result.starred, starCount: result.starCount })
    } catch (err) {
      setError(err instanceof Error ? err.message : '별 처리에 실패했습니다')
    }
  }

  const submitComment = async (event: FormEvent) => {
    event.preventDefault()
    if (!post) return
    if (!user) {
      navigate('/login', { state: { from: `/posts/${post.id}` } })
      return
    }
    setSubmitting(true)
    setError(null)
    try {
      const created = await createComment(post.id, commentText)
      setComments((prev) => [...prev, created])
      setPost({ ...post, commentCount: post.commentCount + 1 })
      setCommentText('')
    } catch (err) {
      setError(err instanceof Error ? err.message : '댓글 작성에 실패했습니다')
    } finally {
      setSubmitting(false)
    }
  }

  const removeComment = async (comment: Comment) => {
    if (!post || !window.confirm('댓글을 삭제할까요?')) return
    try {
      await deleteComment(comment.id)
      setComments((prev) => prev.filter((item) => item.id !== comment.id))
      setPost({ ...post, commentCount: Math.max(0, post.commentCount - 1) })
    } catch (err) {
      setError(err instanceof Error ? err.message : '댓글 삭제에 실패했습니다')
    }
  }

  const removePost = async () => {
    if (!post || !window.confirm('게시물을 삭제할까요? 첨부파일과 댓글도 함께 삭제됩니다.')) return
    try {
      await deletePost(post.id)
      navigate('/')
    } catch (err) {
      setError(err instanceof Error ? err.message : '게시물 삭제에 실패했습니다')
    }
  }

  if (notFound) {
    return (
      <div className="post-detail post-detail--empty">
        <p>게시물을 찾을 수 없습니다.</p>
        <Link to="/">목록으로 돌아가기</Link>
      </div>
    )
  }

  if (!post) {
    return (
      <div className="post-detail post-detail--empty">
        <p>불러오는 중...</p>
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
        <time dateTime={post.createdAt}>{formatDate(post.createdAt)}</time>
        <div className="post-detail__tags">
          {post.tags.map((tag) => (
            <span key={tag} className="post-detail__tag">
              #{tag}
            </span>
          ))}
        </div>
        <div className="post-detail__stats">
          <button
            type="button"
            className={`post-detail__star ${post.starred ? 'is-active' : ''}`}
            onClick={toggleStar}
            aria-pressed={post.starred}
          >
            {post.starred ? '★' : '☆'} {post.starCount}
          </button>
          <span>💬 {post.commentCount}</span>
          {isOwner && (
            <button type="button" className="post-detail__delete" onClick={removePost}>
              삭제
            </button>
          )}
        </div>
      </div>

      <div className="post-detail__body">
        <Markdown content={post.content} />
      </div>

      {post.attachments.length > 0 && (
        <section className="post-detail__attachments">
          <h2>첨부파일</h2>
          <ul>
            {post.attachments.map((attachment) => (
              <li key={attachment.id}>
                <a href={attachment.downloadUrl} download={attachment.originalName}>
                  {attachment.originalName}
                </a>
                <span className="post-detail__attachment-meta">
                  {formatBytes(attachment.size)} · {attachment.hashedName}
                </span>
              </li>
            ))}
          </ul>
        </section>
      )}

      <section className="comments">
        <h2>댓글 {comments.length}</h2>
        {error && <p className="comments__error">{error}</p>}
        {comments.length === 0 ? (
          <p className="comments__empty">첫 댓글을 남겨보세요.</p>
        ) : (
          <ul className="comments__list">
            {comments.map((comment) => {
              const canDelete = user ? user.nickname === comment.author.nickname || isOwner : false
              return (
                <li key={comment.id} className="comment">
                  <div className="comment__meta">
                    <span className="comment__author">{comment.author.nickname}</span>
                    <time dateTime={comment.createdAt}>{formatDateTime(comment.createdAt)}</time>
                    {comment.updatedAt !== comment.createdAt && <span>(수정됨)</span>}
                    {canDelete && (
                      <button type="button" onClick={() => removeComment(comment)}>
                        삭제
                      </button>
                    )}
                  </div>
                  <p className="comment__content">{comment.content}</p>
                </li>
              )
            })}
          </ul>
        )}

        <form className="comment-form" onSubmit={submitComment}>
          <textarea
            value={commentText}
            onChange={(event) => setCommentText(event.target.value)}
            rows={3}
            placeholder={user ? '댓글을 입력하세요' : '댓글을 남기려면 로그인하세요'}
            required
          />
          <button type="submit" disabled={submitting}>
            {user ? '댓글 작성' : '로그인하고 댓글 작성'}
          </button>
        </form>
      </section>
    </article>
  )
}

export default PostDetailPage
