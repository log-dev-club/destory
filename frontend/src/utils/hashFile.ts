import { sha256 } from 'js-sha256'

/**
 * 팀원 참고: crypto.subtle 은 secure context(HTTPS 또는 localhost)에서만 존재한다.
 * 사설 IP + HTTP 로 접속하는 배포 환경(docs/deploy.md 의 "개인 PC, 내부망" 케이스)에서는
 * undefined 이므로, 이때는 js-sha256(순수 JS 구현)으로 대체한다.
 */
async function sha256Hex(data: Uint8Array<ArrayBuffer>): Promise<string> {
  if (typeof crypto !== 'undefined' && crypto.subtle) {
    const digest = await crypto.subtle.digest('SHA-256', data)
    return Array.from(new Uint8Array(digest))
      .map((byte) => byte.toString(16).padStart(2, '0'))
      .join('')
  }
  return sha256(data)
}

/**
 * 릴리즈 파일(zip, exe 등) 첨부 시, 전송 시간과 원본 파일명을 조합해
 * 서버/DB에 전달할 해시된 파일명을 생성한다.
 * DB는 이 해시된 이름을 참조하여 파일을 저장/다운로드한다.
 */
export async function hashReleaseFileName(file: File, sentAt: number = Date.now()): Promise<string> {
  const source = `${sentAt}:${file.name}`
  const encoded = new TextEncoder().encode(source)
  const hashHex = await sha256Hex(encoded)

  const extIndex = file.name.lastIndexOf('.')
  const extension = extIndex >= 0 ? file.name.slice(extIndex) : ''
  return `${hashHex}${extension}`
}

export interface HashedRelease {
  originalName: string
  hashedName: string
  sentAt: number
  size: number
}

export async function buildHashedRelease(file: File): Promise<HashedRelease> {
  const sentAt = Date.now()
  const hashedName = await hashReleaseFileName(file, sentAt)
  return {
    originalName: file.name,
    hashedName,
    sentAt,
    size: file.size,
  }
}
