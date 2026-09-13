// @tiptap/extension-mathematics 는 자체 markdownTokenizer/renderMarkdown 필드를 갖고 있지만,
// 이건 tiptap 코어의 실험적인 네이티브 마크다운 시스템용이라 우리가 쓰는 tiptap-markdown
// 패키지(storage.markdown 규약)에는 적용되지 않는다. 그래서 직렬화만 우리 쪽 규약으로 다시 붙인다.
// 인라인 수식은 `$latex$`, 블록 수식은 독립된 줄에 `$$\nlatex\n$$` 로 저장한다.
import { BlockMath as BaseBlockMath, InlineMath as BaseInlineMath } from '@tiptap/extension-mathematics'

export const InlineMath = BaseInlineMath.extend({
  addStorage() {
    return {
      markdown: {
        serialize(state: { write: (text: string) => void }, node: { attrs: { latex: string } }) {
          state.write(`$${node.attrs.latex}$`)
        },
      },
    }
  },
})

interface BlockSerializerState {
  write: (text: string) => void
  text: (text: string, escape?: boolean) => void
  ensureNewLine: () => void
  closeBlock: (node: unknown) => void
}

export const BlockMath = BaseBlockMath.extend({
  addStorage() {
    return {
      markdown: {
        serialize(state: BlockSerializerState, node: { attrs: { latex: string } }) {
          state.write('$$\n')
          state.text(node.attrs.latex, false)
          state.ensureNewLine()
          state.write('$$')
          state.closeBlock(node)
        },
      },
    }
  },
})
