import type { Editor } from '@tiptap/core'
import { useEditorState } from '@tiptap/react'
import './BubbleToolbar.css'

interface BubbleToolbarProps {
  editor: Editor
}

function BubbleToolbar({ editor }: BubbleToolbarProps) {
  const state = useEditorState({
    editor,
    selector: (snapshot) => ({
      bold: snapshot.editor.isActive('bold'),
      italic: snapshot.editor.isActive('italic'),
      underline: snapshot.editor.isActive('underline'),
      strike: snapshot.editor.isActive('strike'),
      code: snapshot.editor.isActive('code'),
      link: snapshot.editor.isActive('link'),
    }),
  })

  const toggleLink = () => {
    if (state.link) {
      editor.chain().focus().unsetLink().run()
      return
    }
    const url = window.prompt('링크 URL을 입력하세요', 'https://')
    if (!url) return
    editor.chain().focus().extendMarkRange('link').setLink({ href: url }).run()
  }

  return (
    <div className="bubble-toolbar">
      <button
        type="button"
        className={state.bold ? 'is-active' : ''}
        title="굵게"
        onClick={() => editor.chain().focus().toggleBold().run()}
      >
        B
      </button>
      <button
        type="button"
        className={state.italic ? 'is-active' : ''}
        title="기울임"
        onClick={() => editor.chain().focus().toggleItalic().run()}
      >
        I
      </button>
      <button
        type="button"
        className={state.underline ? 'is-active' : ''}
        title="밑줄"
        onClick={() => editor.chain().focus().toggleUnderline().run()}
      >
        U
      </button>
      <button
        type="button"
        className={state.strike ? 'is-active' : ''}
        title="취소선"
        onClick={() => editor.chain().focus().toggleStrike().run()}
      >
        S
      </button>
      <button
        type="button"
        className={state.code ? 'is-active' : ''}
        title="인라인 코드"
        onClick={() => editor.chain().focus().toggleCode().run()}
      >
        {'</>'}
      </button>
      <button type="button" className={state.link ? 'is-active' : ''} title="링크" onClick={toggleLink}>
        🔗
      </button>
    </div>
  )
}

export default BubbleToolbar
