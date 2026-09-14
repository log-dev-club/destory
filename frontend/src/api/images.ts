import { api } from './client'
import type { UploadedImage } from '../types'

/** 본문에 삽입할 이미지를 업로드하고 절대 URL을 돌려받는다 (Discord 썸네일 등 외부 접근용) */
export function uploadImage(file: File) {
  const form = new FormData()
  form.append('file', file)
  return api<UploadedImage>('/api/images', { method: 'POST', body: form })
}
