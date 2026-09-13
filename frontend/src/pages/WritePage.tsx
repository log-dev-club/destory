import { useEffect, useRef, useState } from 'react'
import type { FormEvent } from 'react'
import BlockEditor from '../components/editor/BlockEditor'
import type { BlockEditorHandle } from '../components/editor/BlockEditor'
import { buildHashedRelease } from '../utils/hashFile'
import type { HashedRelease } from '../utils/hashFile'
import { clearWriteDraft, isDraftEmpty, loadWriteDraft, saveWriteDraft } from '../utils/writeDraft'
import type { WriteDraft } from '../utils/writeDraft'
import './WritePage.css'

interface AttachedFile {
  file: File
  hashed: HashedRelease
}

const AUTOSAVE_DELAY_MS = 1200

function formatSavedAt(timestamp: number) {
  return new Date(timestamp).toLocaleTimeString('ko-KR', { hour: '2-digit', minute: '2-digit' })
}

function WritePage() {
  const [title, setTitle] = useState('')
  const [tagsInput, setTagsInput] = useState('')
  const [content, setContent] = useState('')
  const [attachments, setAttachments] = useState<AttachedFile[]>([])
  const [isHashing, setIsHashing] = useState(false)
  const [draftBanner, setDraftBanner] = useState<WriteDraft | null>(null)
  const [draftChecked, setDraftChecked] = useState(false)
  const [lastSavedAt, setLastSavedAt] = useState<number | null>(null)
  const [submitted, setSubmitted] = useState<{
    title: string
    tags: string[]
    content: string
    attachments: HashedRelease[]
  } | null>(null)
  const editorRef = useRef<BlockEditorHandle>(null)

  useEffect(() => {
    const draft = loadWriteDraft()
    if (draft && !isDraftEmpty(draft)) {
      setDraftBanner(draft)
    }
    setDraftChecked(true)
  }, [])

  useEffect(() => {
    if (!draftChecked || draftBanner) return
    if (!title.trim() && !tagsInput.trim() && !content.trim()) return

    const timer = window.setTimeout(() => {
      const saved = saveWriteDraft({ title, tagsInput, content })
      setLastSavedAt(saved.savedAt)
    }, AUTOSAVE_DELAY_MS)

    return () => window.clearTimeout(timer)
  }, [title, tagsInput, content, draftChecked, draftBanner])

  const handleRestoreDraft = () => {
    if (!draftBanner) return
    setTitle(draftBanner.title)
    setTagsInput(draftBanner.tagsInput)
    setContent(draftBanner.content)
    editorRef.current?.setContent(draftBanner.content)
    setLastSavedAt(draftBanner.savedAt)
    setDraftBanner(null)
  }

  const handleDiscardDraft = () => {
    clearWriteDraft()
    setDraftBanner(null)
  }

  const handleSaveDraft = () => {
    const saved = saveWriteDraft({ title, tagsInput, content })
    setLastSavedAt(saved.savedAt)
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

  const handleSubmit = (event: FormEvent) => {
    event.preventDefault()
    if (!title.trim() || !content.trim()) return

    const tags = tagsInput
      .split(',')
      .map((tag) => tag.trim())
      .filter(Boolean)

    setSubmitted({
      title,
      tags,
      content,
      attachments: attachments.map((item) => item.hashed),
    })
    clearWriteDraft()
    setLastSavedAt(null)
  }

  return (
    <div className="write-page">
      {draftBanner && (
        <div className="write-draft-banner">
          <span>{formatSavedAt(draftBanner.savedAt)}에 임시저장된 글이 있습니다.</span>
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

        <div className="write-form__actions">
          <button type="submit" className="write-form__submit" disabled={!title.trim() || !content.trim()}>
            게시하기
          </button>
          <button type="button" className="write-form__draft-save" onClick={handleSaveDraft}>
            임시저장
          </button>
          {lastSavedAt && <span className="write-form__saved-at">{formatSavedAt(lastSavedAt)}에 저장됨</span>}
        </div>
      </form>

      {submitted && (
        <div className="write-preview">
          <h2>서버로 전달될 데이터 미리보기</h2>
          <p>
            <strong>{submitted.title}</strong>
          </p>
          <p>태그: {submitted.tags.join(', ') || '없음'}</p>
          <pre className="write-preview__markdown">{submitted.content}</pre>
          {submitted.attachments.length > 0 && (
            <ul>
              {submitted.attachments.map((attachment) => (
                <li key={attachment.hashedName}>
                  {attachment.originalName} → <code>{attachment.hashedName}</code> (전송시각{' '}
                  {new Date(attachment.sentAt).toLocaleString()})
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  )
}

export default WritePage
