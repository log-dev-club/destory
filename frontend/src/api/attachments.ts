import { api } from './client'
import type { Attachment } from '../types'
import type { HashedRelease } from '../utils/hashFile'

/** buildHashedRelease 로 계산한 hashedName/sentAt 을 파일과 함께 보낸다 (서버가 재검증) */
export function uploadAttachment(postId: number, file: File, hashed: HashedRelease) {
  const form = new FormData()
  form.append('file', file)
  form.append('hashedName', hashed.hashedName)
  form.append('sentAt', String(hashed.sentAt))
  return api<Attachment>(`/api/posts/${postId}/attachments`, { method: 'POST', body: form })
}

export const deleteAttachment = (hashedName: string) =>
  api<void>(`/api/attachments/${hashedName}`, { method: 'DELETE' })
