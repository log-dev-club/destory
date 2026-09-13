const DRAFT_KEY = 'destory:write-draft:v1'

export interface WriteDraft {
  title: string
  tagsInput: string
  content: string
  savedAt: number
}

export function loadWriteDraft(): WriteDraft | null {
  try {
    const raw = localStorage.getItem(DRAFT_KEY)
    if (!raw) return null
    const parsed = JSON.parse(raw) as Partial<WriteDraft>
    if (typeof parsed.title !== 'string' || typeof parsed.tagsInput !== 'string' || typeof parsed.content !== 'string') {
      return null
    }
    return { title: parsed.title, tagsInput: parsed.tagsInput, content: parsed.content, savedAt: parsed.savedAt ?? 0 }
  } catch {
    return null
  }
}

export function saveWriteDraft(draft: Omit<WriteDraft, 'savedAt'>): WriteDraft {
  const full: WriteDraft = { ...draft, savedAt: Date.now() }
  try {
    localStorage.setItem(DRAFT_KEY, JSON.stringify(full))
  } catch {
    // 저장 공간이 없거나 접근이 막힌 경우에는 임시저장을 조용히 건너뜀
  }
  return full
}

export function clearWriteDraft() {
  try {
    localStorage.removeItem(DRAFT_KEY)
  } catch {
    // ignore
  }
}

export function isDraftEmpty(draft: Pick<WriteDraft, 'title' | 'tagsInput' | 'content'>): boolean {
  return !draft.title.trim() && !draft.tagsInput.trim() && !draft.content.trim()
}
