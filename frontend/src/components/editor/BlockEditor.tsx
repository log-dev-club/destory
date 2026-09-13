import { forwardRef, useImperativeHandle } from 'react'
import type { Editor } from '@tiptap/core'
import { NodeSelection } from '@tiptap/pm/state'
import { EditorContent, useEditor } from '@tiptap/react'
import { BubbleMenu } from '@tiptap/react/menus'
import { migrateMathStrings } from '@tiptap/extension-mathematics'
import StarterKit from '@tiptap/starter-kit'
import Placeholder from '@tiptap/extension-placeholder'
import ImageExtension from '@tiptap/extension-image'
import TaskList from '@tiptap/extension-task-list'
import TaskItem from '@tiptap/extension-task-item'
import { TableKit } from '@tiptap/extension-table'
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight'
import { createLowlight, common } from 'lowlight'
import { Markdown } from 'tiptap-markdown'
import type { MarkdownStorage } from 'tiptap-markdown'
import GlobalDragHandle from 'tiptap-extension-global-drag-handle'
import { InlineMath, BlockMath } from './mathExtensions'
import SlashCommand from './SlashCommand'
import BubbleToolbar from './BubbleToolbar'
import 'highlight.js/styles/github-dark.css'
import 'katex/dist/katex.min.css'
import './BlockEditor.css'

const lowlight = createLowlight(common)

function getMarkdown(editor: Editor | null): string {
  return (editor?.storage as { markdown?: MarkdownStorage } | undefined)?.markdown?.getMarkdown() ?? ''
}

export interface BlockEditorHandle {
  getMarkdown: () => string
  setContent: (markdown: string) => void
}

interface BlockEditorProps {
  onChange?: (markdown: string) => void
}

const BlockEditor = forwardRef<BlockEditorHandle, BlockEditorProps>(function BlockEditor({ onChange }, ref) {
  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        codeBlock: false,
        link: { openOnClick: false, autolink: true },
      }),
      Placeholder.configure({
        placeholder: ({ node }) => {
          if (node.type.name === 'heading') return '제목'
          return "내용을 입력하거나 '/'를 눌러 명령어를 사용하세요"
        },
        includeChildren: true,
      }),
      ImageExtension,
      TaskList,
      TaskItem.configure({ nested: true }),
      TableKit.configure({ table: { resizable: false } }),
      CodeBlockLowlight.configure({ lowlight }),
      InlineMath.configure({ katexOptions: { throwOnError: false } }),
      BlockMath.configure({ katexOptions: { throwOnError: false, displayMode: true } }),
      Markdown.configure({ html: false, transformPastedText: true, transformCopiedText: true }),
      GlobalDragHandle.configure({ dragHandleWidth: 24 }),
      SlashCommand,
    ],
    content: '',
    editorProps: {
      attributes: {
        class: 'tiptap block-editor__content markdown-body',
      },
    },
    onCreate: ({ editor }) => {
      // 마크다운 원문의 `$latex$` 인라인 수식 텍스트를 실제 렌더링되는 수식 노드로 변환
      migrateMathStrings(editor)
    },
    onUpdate: ({ editor }) => {
      onChange?.(getMarkdown(editor))
    },
  })

  useImperativeHandle(
    ref,
    () => ({
      getMarkdown: () => getMarkdown(editor),
      setContent: (markdown: string) => {
        editor?.commands.setContent(markdown)
        if (editor) migrateMathStrings(editor)
        onChange?.(getMarkdown(editor))
      },
    }),
    [editor, onChange],
  )

  if (!editor) return null

  return (
    <div className="block-editor">
      <BubbleMenu
        editor={editor}
        shouldShow={({ editor, state }) => {
          const { selection } = state
          if (selection.empty || selection instanceof NodeSelection) return false
          return editor.isEditable
        }}
      >
        <BubbleToolbar editor={editor} />
      </BubbleMenu>
      <EditorContent editor={editor} />
    </div>
  )
})

export default BlockEditor
