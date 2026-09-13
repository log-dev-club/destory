import type { PostListParams } from '../api/posts'

export interface ParsedSearchQuery {
  tags: string[]
  users: string[]
  text: string[]
}

const PREFIX_PATTERN = /^(tag|user):(.+)$/i

/** 검색창 입력을 tag:/user:/자유 텍스트로 분리 */
export function parseSearchQuery(query: string): ParsedSearchQuery {
  const tags: string[] = []
  const users: string[] = []
  const text: string[] = []

  for (const token of query.trim().split(/\s+/).filter(Boolean)) {
    const match = token.match(PREFIX_PATTERN)
    if (!match) {
      text.push(token.toLowerCase())
      continue
    }

    const [, prefix, value] = match
    if (!value) continue

    if (prefix.toLowerCase() === 'tag') {
      tags.push(value.toLowerCase())
    } else {
      users.push(value.toLowerCase())
    }
  }

  return { tags, users, text }
}

/** 파싱 결과를 GET /api/posts 쿼리 파라미터로 변환 */
export function toPostListParams(parsed: ParsedSearchQuery): PostListParams {
  return {
    tag: parsed.tags.join(',') || undefined,
    user: parsed.users[0],
    q: parsed.text.join(' ') || undefined,
  }
}

export function isEmptyQuery(parsed: ParsedSearchQuery): boolean {
  return parsed.tags.length === 0 && parsed.users.length === 0 && parsed.text.length === 0
}
