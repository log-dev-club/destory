import ReactMarkdown, { defaultUrlTransform } from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkMath from 'remark-math'
import rehypeHighlight from 'rehype-highlight'
import rehypeKatex from 'rehype-katex'
import 'highlight.js/styles/github-dark.css'
import 'katex/dist/katex.min.css'
import './Markdown.css'

interface MarkdownProps {
  content: string
}

// react-markdown 은 XSS 방지를 위해 http(s)/mailto 등만 허용하고 data: URI는 기본 차단한다.
// 첨부 이미지를 base64 데이터 URI로 저장하므로, 이미지 데이터 URI만 예외로 허용한다.
const SAFE_IMAGE_DATA_URI = /^data:image\/(png|jpe?g|gif|webp|svg\+xml);base64,/i

function urlTransform(url: string) {
  if (SAFE_IMAGE_DATA_URI.test(url)) return url
  return defaultUrlTransform(url)
}

function Markdown({ content }: MarkdownProps) {
  return (
    <div className="markdown-body">
      <ReactMarkdown
        remarkPlugins={[remarkGfm, remarkMath]}
        rehypePlugins={[rehypeHighlight, rehypeKatex]}
        urlTransform={urlTransform}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}

export default Markdown
