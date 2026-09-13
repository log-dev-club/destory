import type { Post } from '../types'

export interface ParsedSearchQuery {
  tags: string[]
  users: string[]
  text: string[]
}

const PREFIX_PATTERN = /^(tag|user):(.+)$/i

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

export function matchesSearchQuery(post: Post, parsed: ParsedSearchQuery): boolean {
  const { tags, users, text } = parsed

  if (tags.length > 0) {
    const postTags = post.tags.map((tag) => tag.toLowerCase())
    if (!tags.every((tag) => postTags.some((postTag) => postTag.includes(tag)))) {
      return false
    }
  }

  if (users.length > 0) {
    const nickname = post.author.nickname.toLowerCase()
    if (!users.every((user) => nickname.includes(user))) {
      return false
    }
  }

  if (text.length > 0) {
    const haystack = [post.title, post.excerpt, post.author.nickname, ...post.tags]
      .join(' ')
      .toLowerCase()
    if (!text.every((word) => haystack.includes(word))) {
      return false
    }
  }

  return true
}
