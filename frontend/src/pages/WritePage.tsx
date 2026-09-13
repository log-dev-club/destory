import { useEffect, useRef, useState } from 'react'
import type { FormEvent } from 'react'
import { useNavigate, useOutletContext } from 'react-router-dom'
import Markdown from '../components/Markdown'
import type { LayoutContext } from '../components/Layout'
import { createPost } from '../api/posts'
import { uploadAttachment } from '../api/attachments'
import { buildHashedRelease } from '../utils/hashFile'
import type { HashedRelease } from '../utils/hashFile'
import './WritePage.css'

interface AttachedFile {
  file: File
  hashed: HashedRelease
}

interface ToolbarAction {
  label: string
  title: string
  apply: (value: string, selectionStart: number, selectionEnd: number) => {
    value: string
    selectionStart: number
    selectionEnd: number
  }
}

function wrapSelection(before: string, after: string, placeholder: string) {
  return (value: string, selectionStart: number, selectionEnd: number) => {
    const selected = value.slice(selectionStart, selectionEnd) || placeholder
    const nextValue = value.slice(0, selectionStart) + before + selected + after + value.slice(selectionEnd)
    return {
      value: nextValue,
      selectionStart: selectionStart + before.length,
      selectionEnd: selectionStart + before.length + selected.length,
    }
  }
}

function prefixLine(prefix: string) {
  return (value: string, selectionStart: number, selectionEnd: number) => {
    const lineStart = value.lastIndexOf('\n', selectionStart - 1) + 1
    const nextValue = value.slice(0, lineStart) + prefix + value.slice(lineStart)
    return {
      value: nextValue,
      selectionStart: selectionStart + prefix.length,
      selectionEnd: selectionEnd + prefix.length,
    }
  }
}

const TOOLBAR_ACTIONS: ToolbarAction[] = [
  { label: 'H', title: '제목', apply: prefixLine('## ') },
  { label: 'B', title: '굵게', apply: wrapSelection('**', '**', '굵은 텍스트') },
  { label: 'I', title: '기울임', apply: wrapSelection('*', '*', '기울임 텍스트') },
  { label: '`code`', title: '인라인 코드', apply: wrapSelection('`', '`', 'code') },
  {
    label: '{ }',
    title: '코드 블록',
    apply: wrapSelection('\n```\n', '\n```\n', '여기에 코드를 입력하세요'),
  },
  { label: '"', title: '인용', apply: prefixLine('> ') },
  { label: '•', title: '목록', apply: prefixLine('- ') },
  { label: '🔗', title: '링크', apply: wrapSelection('[', '](https://)', '링크 텍스트') },
]

function WritePage() {
  const { user } = useOutletContext<LayoutContext>()
  const navigate = useNavigate()

  const [title, setTitle] = useState('')
  const [tagsInput, setTagsInput] = useState('')
  const [content, setContent] = useState('')
  const [activeTab, setActiveTab] = useState<'write' | 'preview'>('write')
  const [attachments, setAttachments] = useState<AttachedFile[]>([])
  const [isHashing, setIsHashing] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  // 비로그인이면 로그인 후 돌아오도록
  useEffect(() => {
    if (user === null) navigate('/login', { replace: true, state: { from: '/write' } })
  }, [user, navigate])

  const runToolbarAction = (action: ToolbarAction) => {
    const textarea = textareaRef.current
    if (!textarea) return

    const result = action.apply(content, textarea.selectionStart, textarea.selectionEnd)
    setContent(result.value)
    requestAnimationFrame(() => {
      textarea.focus()
      textarea.setSelectionRange(result.selectionStart, result.selectionEnd)
    })
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
   * 2) 응답 id 로 첨부파일을 하나씩 업로드 (hashedName/sentAt 은 서버가 재검증)
   * 3) 상세 화면으로 이동
   */
  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault()
    if (submitting || isHashing) return
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
      navigate(`/posts/${post.id}`)
    } catch (err) {
      setError(err instanceof Error ? err.message : '게시물 저장에 실패했습니다')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="write-page">
      <h1>새 게시물 작성</h1>
      <form className="write-form" onSubmit={handleSubmit}>
        <label className="write-form__field">
          <span>제목</span>
          <input
            type="text"
            value={title}
            onChange={(event) => setTitle(event.target.value)}
            placeholder="게시물 제목을 입력하세요"
            required
          />
        </label>

        <label className="write-form__field">
          <span>태그</span>
          <input
            type="text"
            value={tagsInput}
            onChange={(event) => setTagsInput(event.target.value)}
            placeholder="쉼표(,)로 구분해서 입력하세요 (예: react, frontend)"
          />
        </label>

        <div className="write-form__field">
          <span>내용 (마크다운 지원)</span>
          <div className="markdown-editor">
            <div className="markdown-editor__toolbar">
              {TOOLBAR_ACTIONS.map((action) => (
                <button
                  key={action.title}
                  type="button"
                  title={action.title}
                  onClick={() => runToolbarAction(action)}
                >
                  {action.label}
                </button>
              ))}
              <div className="markdown-editor__tabs">
                <button
                  type="button"
                  className={activeTab === 'write' ? 'is-active' : ''}
                  onClick={() => setActiveTab('write')}
                >
                  작성
                </button>
                <button
                  type="button"
                  className={activeTab === 'preview' ? 'is-active' : ''}
                  onClick={() => setActiveTab('preview')}
                >
                  미리보기
                </button>
              </div>
            </div>

            {activeTab === 'write' ? (
              <textarea
                ref={textareaRef}
                className="markdown-editor__textarea"
                value={content}
                onChange={(event) => setContent(event.target.value)}
                rows={14}
                placeholder="마크다운으로 내용을 작성하세요. 코드 블록은 ``` 로 감싸주세요."
                required
              />
            ) : (
              <div className="markdown-editor__preview">
                {content ? (
                  <Markdown content={content} />
                ) : (
                  <p className="markdown-editor__preview-empty">미리볼 내용이 없습니다.</p>
                )}
              </div>
            )}
          </div>
        </div>

        <div className="write-form__field">
          <span>릴리즈 파일 첨부 (zip, exe 등)</span>
          <input
            type="file"
            multiple
            onChange={(event) => handleFilesSelected(event.target.files)}
          />
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

        <button type="submit" className="write-form__submit" disabled={submitting || isHashing}>
          {submitting ? '저장 중...' : '게시하기'}
        </button>
      </form>
    </div>
  )
}

export default WritePage
