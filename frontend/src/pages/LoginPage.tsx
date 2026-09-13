import { useState } from 'react'
import type { FormEvent } from 'react'
import { useLocation, useNavigate, useOutletContext } from 'react-router-dom'
import { login, register } from '../api/auth'
import type { LayoutContext } from '../components/Layout'
import { PASSWORD_MIN_LENGTH, hasSpecialChar, validatePassword } from '../utils/password'
import './LoginPage.css'

type Mode = 'login' | 'register'

function LoginPage() {
  const { setUser } = useOutletContext<LayoutContext>()
  const navigate = useNavigate()
  const location = useLocation()
  const redirectTo = (location.state as { from?: string } | null)?.from ?? '/'

  const [mode, setMode] = useState<Mode>('login')
  const [nickname, setNickname] = useState('')
  const [password, setPassword] = useState('')
  const [passwordConfirm, setPasswordConfirm] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  const isRegister = mode === 'register'
  const lengthOk = [...password].length >= PASSWORD_MIN_LENGTH
  const specialOk = hasSpecialChar(password)
  const confirmOk = passwordConfirm.length > 0 && password === passwordConfirm

  const switchMode = () => {
    setMode(isRegister ? 'login' : 'register')
    setPasswordConfirm('')
    setError(null)
  }

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault()
    setError(null)

    if (isRegister) {
      // 서버에도 같은 규칙이 있지만, 왕복 없이 바로 알려주기 위해 먼저 검사한다
      const message = validatePassword(password)
      if (message) {
        setError(message)
        return
      }
      if (password !== passwordConfirm) {
        setError('비밀번호가 일치하지 않습니다')
        return
      }
    }

    setSubmitting(true)
    try {
      const action = isRegister ? register : login
      const user = await action({ nickname: nickname.trim(), password })
      setUser(user)
      navigate(redirectTo, { replace: true })
    } catch (err) {
      setError(err instanceof Error ? err.message : '요청에 실패했습니다')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="login-page">
      <h1>{isRegister ? '회원가입' : '로그인'}</h1>
      <form className="login-form" onSubmit={handleSubmit}>
        <label className="login-form__field">
          <span>닉네임</span>
          <input
            type="text"
            value={nickname}
            onChange={(event) => setNickname(event.target.value)}
            autoComplete="username"
            placeholder="2~20자, 문자·숫자·_·-"
            required
          />
        </label>
        <label className="login-form__field">
          <span>비밀번호</span>
          <input
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            autoComplete={isRegister ? 'new-password' : 'current-password'}
            placeholder={isRegister ? '8자 이상, 특수문자 포함' : ''}
            required
          />
        </label>

        {isRegister && (
          <>
            <ul className="login-form__rules" aria-live="polite">
              <li className={lengthOk ? 'is-ok' : ''}>{PASSWORD_MIN_LENGTH}자 이상</li>
              <li className={specialOk ? 'is-ok' : ''}>특수문자(!@#$%^&* 등) 1개 이상</li>
              <li className={confirmOk ? 'is-ok' : ''}>비밀번호 확인 일치</li>
            </ul>
            <label className="login-form__field">
              <span>비밀번호 확인</span>
              <input
                type="password"
                value={passwordConfirm}
                onChange={(event) => setPasswordConfirm(event.target.value)}
                autoComplete="new-password"
                required
                aria-invalid={passwordConfirm.length > 0 && !confirmOk}
              />
            </label>
          </>
        )}

        {error && <p className="login-form__error">{error}</p>}
        <button type="submit" className="login-form__submit" disabled={submitting}>
          {isRegister ? '가입하기' : '로그인'}
        </button>
      </form>
      <p className="login-page__switch">
        {isRegister ? '이미 계정이 있나요?' : '계정이 없나요?'}{' '}
        <button type="button" onClick={switchMode}>
          {isRegister ? '로그인' : '회원가입'}
        </button>
      </p>
    </div>
  )
}

export default LoginPage
