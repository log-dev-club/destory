import { api } from './client'

export interface Draft {
  title: string
  content: string
  tagsInput: string
  updatedAt: string
}

export const fetchDraft = () => api<Draft | null>('/api/users/me/draft')

export interface SaveDraftBody {
  title: string
  content: string
  tagsInput: string
}

export const saveDraft = (body: SaveDraftBody) =>
  api<Draft>('/api/users/me/draft', { method: 'PUT', body: JSON.stringify(body) })

export const deleteDraft = () => api<void>('/api/users/me/draft', { method: 'DELETE' })
