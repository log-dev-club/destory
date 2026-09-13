import { useEffect, useState } from 'react'
import type { FormEvent } from 'react'
import { useLocation, useNavigate, useOutletContext, useSearchParams } from 'react-router-dom'
import { fetchProviders, login, register, startGithubLogin } from '../api/auth'
import type { LayoutContext } from '../components/Layout'
import { PASSWORD_MIN_LENGTH, hasSpecialChar, validatePassword } from '../utils/password'
import './LoginPage.css'

type Mode = 'login' | 'register'

function LoginPage() {
  const { setUser } = useOutletContext<LayoutContext>()
  const navigate = useNavigate()
  const location = useLocation()
  const [searchParams] = useSearchParams()
  const redirectTo = (location.state as { from?: string } | null)?.from ?? '/'

  const [mode, setMode] = useState<Mode>('login')
  const [nickname, setNickname] = useState('')
  const [password, setPassword] = useState('')
  const [passwordConfirm, setPasswordConfirm] = useState('')
  // GitHub 콜백이 실패하면 /login?error=메시지 로 돌아온다
  const [error, setError] = useState<string | null>(() => searchParams.get('error'))
  const [submitting, setSubmitting] = useState(false)
  const [githubEnabled, setGithubEnabled] = useState(false)

  useEffect(() => {
    fetchProviders()
      .then((providers) => setGithubEnabled(providers.github))
      .catch(() => setGithubEnabled(false))
  }, [])

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

      {githubEnabled && (
        <>
          <div className="login-page__divider">또는</div>
          <button type="button" className="login-form__github" onClick={startGithubLogin}>
            <svg viewBox="0 0 16 16" width="18" height="18" aria-hidden="true">
              <path
                fill="currentColor"
                d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8"
              />
            </svg>
            GitHub로 {isRegister ? '가입하기' : '로그인'}
          </button>
        </>
      )}
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
