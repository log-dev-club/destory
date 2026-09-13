import { useEffect, useRef, useState } from 'react'
import type { FormEvent } from 'react'
import { useNavigate, useOutletContext } from 'react-router-dom'
import BlockEditor from '../components/editor/BlockEditor'
import type { BlockEditorHandle } from '../components/editor/BlockEditor'
import type { LayoutContext } from '../components/Layout'
import { createPost } from '../api/posts'
import { uploadAttachment } from '../api/attachments'
import { deleteDraft, fetchDraft, saveDraft } from '../api/drafts'
import type { Draft } from '../api/drafts'
import { ApiError } from '../api/client'
import { buildHashedRelease } from '../utils/hashFile'
import type { HashedRelease } from '../utils/hashFile'
import './WritePage.css'

interface AttachedFile {
  file: File
  hashed: HashedRelease
}

const AUTOSAVE_DELAY_MS = 1500

function formatSavedAt(iso: string) {
  return new Date(iso).toLocaleTimeString('ko-KR', { hour: '2-digit', minute: '2-digit' })
}

function isDraftEmpty(draft: Pick<Draft, 'title' | 'tagsInput' | 'content'>) {
  return !draft.title.trim() && !draft.tagsInput.trim() && !draft.content.trim()
}

function WritePage() {
  const { user } = useOutletContext<LayoutContext>()
  const navigate = useNavigate()

  const [title, setTitle] = useState('')
  const [tagsInput, setTagsInput] = useState('')
  const [content, setContent] = useState('')
  const [attachments, setAttachments] = useState<AttachedFile[]>([])
  const [isHashing, setIsHashing] = useState(false)
  const [draftBanner, setDraftBanner] = useState<Draft | null>(null)
  const [draftChecked, setDraftChecked] = useState(false)
  const [lastSavedAt, setLastSavedAt] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const editorRef = useRef<BlockEditorHandle>(null)

  // 비로그인이면 로그인 후 이 페이지로 돌아오도록
  useEffect(() => {
    if (user === null) navigate('/login', { replace: true, state: { from: '/write' } })
  }, [user, navigate])

  // 로그인 확인되면 서버에 저장된 임시저장이 있는지 확인
  useEffect(() => {
    if (!user) return
    let cancelled = false
    fetchDraft()
      .then((draft) => {
        if (!cancelled && draft && !isDraftEmpty(draft)) setDraftBanner(draft)
      })
      .catch(() => {})
      .finally(() => {
        if (!cancelled) setDraftChecked(true)
      })
    return () => {
      cancelled = true
    }
  }, [user])

  // 자동 임시저장 (디바운스). 복원 배너가 떠 있는 동안엔 덮어쓰지 않는다
  useEffect(() => {
    if (!user || !draftChecked || draftBanner) return
    if (!title.trim() && !tagsInput.trim() && !content.trim()) return

    const timer = window.setTimeout(() => {
      saveDraft({ title, content, tagsInput })
        .then((saved) => setLastSavedAt(saved.updatedAt))
        .catch(() => {})
    }, AUTOSAVE_DELAY_MS)

    return () => window.clearTimeout(timer)
  }, [user, title, tagsInput, content, draftChecked, draftBanner])

  const handleRestoreDraft = () => {
    if (!draftBanner) return
    setTitle(draftBanner.title)
    setTagsInput(draftBanner.tagsInput)
    setContent(draftBanner.content)
    editorRef.current?.setContent(draftBanner.content)
    setLastSavedAt(draftBanner.updatedAt)
    setDraftBanner(null)
  }

  const handleDiscardDraft = () => {
    setDraftBanner(null)
    deleteDraft().catch(() => {})
  }

  const handleSaveDraft = () => {
    saveDraft({ title, content, tagsInput })
      .then((saved) => setLastSavedAt(saved.updatedAt))
      .catch((err: unknown) => setError(err instanceof ApiError ? err.message : '임시저장에 실패했습니다'))
  }

  const handleFilesSelected = async (fileList: FileList | null) => {
    if (!fileList || fileList.length === 0) return
    setIsHashing(true)
    try {
      const newAttachments = await Promise.all(
        Array.from(fileList).map(async (file) => ({
          file,
          hashed: await buildHashedRelease(file),
        })),
      )
      setAttachments((prev) => [...prev, ...newAttachments])
    } finally {
      setIsHashing(false)
    }
  }

  const removeAttachment = (hashedName: string) => {
    setAttachments((prev) => prev.filter((item) => item.hashed.hashedName !== hashedName))
  }

  /**
   * 1) POST /api/posts 로 게시물 저장 (서버가 Discord 알림 전송)
   * 2) 응답 id 로 첨부파일을 하나씩 업로드
   * 3) 임시저장 삭제 후 상세 화면으로 이동
   */
  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault()
    if (submitting || isHashing) return
    if (!title.trim() || !content.trim()) return

    const tags = tagsInput
      .split(',')
      .map((tag) => tag.trim())
      .filter(Boolean)

    setSubmitting(true)
    setError(null)
    try {
      const post = await createPost({ title, content, tags })

      const failed: string[] = []
      for (const { file, hashed } of attachments) {
        try {
          await uploadAttachment(post.id, file, hashed)
        } catch (err) {
          failed.push(`${file.name}: ${err instanceof Error ? err.message : '업로드 실패'}`)
        }
      }
      if (failed.length > 0) {
        window.alert(`게시물은 저장되었지만 일부 첨부파일 업로드에 실패했습니다.\n${failed.join('\n')}`)
      }

      await deleteDraft().catch(() => {})
      navigate(`/posts/${post.id}`)
    } catch (err) {
      setError(err instanceof ApiError ? err.message : '게시물 저장에 실패했습니다')
    } finally {
      setSubmitting(false)
    }
  }

  // 로그인 확인 전이거나 비로그인으로 리다이렉트되는 중에는 그리지 않는다
  if (!user) return null

  return (
    <div className="write-page">
      {draftBanner && (
        <div className="write-draft-banner">
          <span>{formatSavedAt(draftBanner.updatedAt)}에 임시저장된 글이 있습니다.</span>
          <div className="write-draft-banner__actions">
            <button type="button" onClick={handleRestoreDraft}>
              불러오기
            </button>
            <button type="button" onClick={handleDiscardDraft}>
              삭제
            </button>
          </div>
        </div>
      )}

      <form className="write-form" onSubmit={handleSubmit}>
        <input
          className="write-form__title"
          type="text"
          value={title}
          onChange={(event) => setTitle(event.target.value)}
          placeholder="제목을 입력하세요"
          required
        />

        <input
          className="write-form__tags"
          type="text"
          value={tagsInput}
          onChange={(event) => setTagsInput(event.target.value)}
          placeholder="태그 추가 (쉼표로 구분, 예: react, frontend)"
        />

        <div className="write-form__editor">
          <BlockEditor ref={editorRef} onChange={setContent} />
        </div>

        <div className="write-form__attachments">
          <label className="write-form__attachments-label">
            릴리즈 파일 첨부 (zip, exe 등)
            <input type="file" multiple onChange={(event) => handleFilesSelected(event.target.files)} />
          </label>
          {isHashing && <p className="write-form__hint">파일 이름을 해싱하는 중...</p>}

          {attachments.length > 0 && (
            <ul className="attachment-list">
              {attachments.map(({ file, hashed }) => (
                <li key={hashed.hashedName} className="attachment-list__item">
                  <div className="attachment-list__info">
                    <span className="attachment-list__name">{file.name}</span>
                    <span className="attachment-list__hash">→ {hashed.hashedName}</span>
                  </div>
                  <button type="button" onClick={() => removeAttachment(hashed.hashedName)}>
                    제거
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>

        {error && <p className="write-form__error">{error}</p>}

        <div className="write-form__actions">
          <button
            type="submit"
            className="write-form__submit"
            disabled={submitting || isHashing || !title.trim() || !content.trim()}
          >
            {submitting ? '게시 중...' : '게시하기'}
          </button>
          <button type="button" className="write-form__draft-save" onClick={handleSaveDraft} disabled={submitting}>
            임시저장
          </button>
          {lastSavedAt && <span className="write-form__saved-at">{formatSavedAt(lastSavedAt)}에 저장됨</span>}
        </div>
      </form>
    </div>
  )
}

export default WritePage
