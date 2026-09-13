import type { Editor, Range } from '@tiptap/core'

export interface SlashCommandItem {
  title: string
  description: string
  icon: string
  keywords: string[]
  command: (params: { editor: Editor; range: Range }) => void
}

const ITEMS: SlashCommandItem[] = [
  {
    title: '텍스트',
    description: '일반 텍스트 문단',
    icon: 'T',
    keywords: ['text', 'paragraph', '텍스트', '문단'],
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).setParagraph().run()
    },
  },
  {
    title: '제목 1',
    description: '가장 큰 섹션 제목',
    icon: 'H1',
    keywords: ['heading', 'h1', '제목'],
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).setNode('heading', { level: 1 }).run()
    },
  },
  {
    title: '제목 2',
    description: '중간 섹션 제목',
    icon: 'H2',
    keywords: ['heading', 'h2', '제목'],
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).setNode('heading', { level: 2 }).run()
    },
  },
  {
    title: '제목 3',
    description: '작은 섹션 제목',
    icon: 'H3',
    keywords: ['heading', 'h3', '제목'],
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).setNode('heading', { level: 3 }).run()
    },
  },
  {
    title: '글머리 기호 목록',
    description: '순서 없는 목록 만들기',
    icon: '•',
    keywords: ['bullet', 'list', '목록', '글머리'],
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).toggleBulletList().run()
    },
  },
  {
    title: '번호 매기기 목록',
    description: '순서 있는 목록 만들기',
    icon: '1.',
    keywords: ['numbered', 'ordered', 'list', '번호', '목록'],
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).toggleOrderedList().run()
    },
  },
  {
    title: '할 일 목록',
    description: '체크박스가 있는 목록',
    icon: '☑',
    keywords: ['todo', 'task', 'checklist', '체크리스트', '할일'],
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).toggleTaskList().run()
    },
  },
  {
    title: '인용',
    description: '인용구 블록',
    icon: '❝',
    keywords: ['quote', 'blockquote', '인용'],
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).toggleBlockquote().run()
    },
  },
  {
    title: '코드 블록',
    description: '문법 강조가 적용된 코드',
    icon: '</>',
    keywords: ['code', '코드'],
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).toggleCodeBlock().run()
    },
  },
  {
    title: '구분선',
    description: '섹션을 나누는 가로줄',
    icon: '—',
    keywords: ['divider', 'hr', 'line', '구분선'],
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).setHorizontalRule().run()
    },
  },
  {
    title: '표',
    description: '3x3 표 삽입',
    icon: '▦',
    keywords: ['table', '표', '테이블'],
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()
    },
  },
  {
    title: '이미지',
    description: '내 컴퓨터에서 이미지 파일 선택',
    icon: '🖼',
    keywords: ['image', 'img', '이미지', '그림', '사진'],
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).run()
      pickImageFile((dataUrl, file) => {
        editor.chain().focus().setImage({ src: dataUrl, alt: file.name }).run()
      })
    },
  },
]

function pickImageFile(onLoaded: (dataUrl: string, file: File) => void) {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = 'image/*'
  input.style.display = 'none'
  input.addEventListener('change', () => {
    const file = input.files?.[0]
    if (file) {
      const reader = new FileReader()
      reader.onload = () => {
        if (typeof reader.result === 'string') onLoaded(reader.result, file)
      }
      reader.readAsDataURL(file)
    }
    input.remove()
  })
  document.body.appendChild(input)
  input.click()
}

export function getSlashCommandItems(query: string): SlashCommandItem[] {
  const q = query.toLowerCase().trim()
  if (!q) return ITEMS.slice(0, 10)
  return ITEMS.filter(
    (item) =>
      item.title.toLowerCase().includes(q) ||
      item.keywords.some((keyword) => keyword.toLowerCase().includes(q) || q.includes(keyword.toLowerCase())),
  ).slice(0, 10)
}
