/**
 * 비밀번호 규칙. 백엔드 services/auth.rs 의 validate_password 와 동일하게 유지한다.
 * - 8자 이상
 * - 공백 불가
 * - 특수문자(문자·숫자·공백이 아닌 문자) 1개 이상
 */
export const PASSWORD_MIN_LENGTH = 8

const SPECIAL_CHAR = /[^\p{L}\p{N}\s]/u

export function hasSpecialChar(password: string): boolean {
  return SPECIAL_CHAR.test(password)
}

/** 규칙 위반 시 사용자에게 보여줄 메시지, 통과하면 null */
export function validatePassword(password: string): string | null {
  if ([...password].length < PASSWORD_MIN_LENGTH) {
    return `비밀번호는 ${PASSWORD_MIN_LENGTH}자 이상이어야 합니다`
  }
  if (/\s/.test(password)) {
    return '비밀번호에 공백을 포함할 수 없습니다'
  }
  if (!hasSpecialChar(password)) {
    return '비밀번호에 특수문자(!@#$%^&* 등)를 1개 이상 포함해야 합니다'
  }
  return null
}
