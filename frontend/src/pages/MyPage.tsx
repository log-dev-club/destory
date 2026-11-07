import { useEffect, useState } from 'react'
import type { FormEvent } from 'react'
import { useNavigate, useOutletContext } from 'react-router-dom'
import PostCard from '../components/PostCard'
import type { LayoutContext } from '../components/Layout'
import { logout } from '../api/auth'
import { fetchMyPosts } from '../api/posts'
import { changePassword, updateMyProfile } from '../api/users'
import { PASSWORD_MIN_LENGTH, hasSpecialChar, validatePassword } from '../utils/password'
import type { PostPage } from '../types'
import './MyPage.css'

function MyPage() {
  const { user, setUser } = useOutletContext<LayoutContext>()
  const navigate = useNavigate()

  const [posts, setPosts] = useState<PostPage | null>(null)
  const [editing, setEditing] = useState(false)
  const [nickname, setNickname] = useState('')
  const [avatarUrl, setAvatarUrl] = useState('')
  const [bio, setBio] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  const [changingPassword, setChangingPassword] = useState(false)
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [newPasswordConfirm, setNewPasswordConfirm] = useState('')
  const [passwordError, setPasswordError] = useState<string | null>(null)
  const [passwordSaving, setPasswordSaving] = useState(false)
  const [passwordChanged, setPasswordChanged] = useState(false)

  // 비로그인이면 로그인 화면으로
  useEffect(() => {
    if (user === null) navigate('/login', { replace: true, state: { from: '/me' } })
  }, [user, navigate])

  useEffect(() => {
    if (!user) return
    fetchMyPosts()
      .then(setPosts)
      .catch((err: Error) => setError(err.message))
  }, [user])

  const startEditing = () => {
    if (!user) return
    setNickname(user.nickname)
    setAvatarUrl(user.avatarUrl ?? '')
    setBio(user.bio ?? '')
    setError(null)
    setEditing(true)
  }

  const saveProfile = async (event: FormEvent) => {
    event.preventDefault()
    if (!user) return
    setSaving(true)
    setError(null)
    try {
      const updated = await updateMyProfile({
        nickname: nickname.trim() !== user.nickname ? nickname.trim() : undefined,
        avatarUrl: avatarUrl.trim(),
        bio: bio.trim(),
      })
      setUser(updated)
      setEditing(false)
      // 닉네임이 바뀌면 목록의 author 도 바뀌므로 다시 조회
      setPosts(await fetchMyPosts())
    } catch (err) {
      setError(err instanceof Error ? err.message : '저장에 실패했습니다')
    } finally {
      setSaving(false)
    }
  }

  const startChangingPassword = () => {
    setCurrentPassword('')
    setNewPassword('')
    setNewPasswordConfirm('')
    setPasswordError(null)
    setPasswordChanged(false)
    setChangingPassword(true)
  }

  const submitPasswordChange = async (event: FormEvent) => {
    event.preventDefault()
    setPasswordError(null)

    const message = validatePassword(newPassword)
    if (message) {
      setPasswordError(message)
      return
    }
    if (newPassword !== newPasswordConfirm) {
      setPasswordError('새 비밀번호가 일치하지 않습니다')
      return
    }

    setPasswordSaving(true)
    try {
      await changePassword(currentPassword, newPassword)
      setChangingPassword(false)
      setPasswordChanged(true)
    } catch (err) {
      setPasswordError(err instanceof Error ? err.message : '비밀번호 변경에 실패했습니다')
    } finally {
      setPasswordSaving(false)
    }
  }

  const handleLogout = async () => {
    try {
      await logout()
    } finally {
      setUser(null)
      navigate('/')
    }
  }

  if (!user) {
    return (
      <div className="my-page">
        <p className="my-page__empty">불러오는 중...</p>
      </div>
    )
  }

  return (
    <div className="my-page">
      <div className="my-page__profile">
        <div className="my-page__avatar">
          {user.avatarUrl ? (
            <img src={user.avatarUrl} alt="" width="72" height="72" style={{ borderRadius: '50%' }} />
          ) : (
            <svg viewBox="0 0 24 24" width="48" height="48" aria-hidden="true">
              <circle cx="12" cy="8" r="4" fill="currentColor" />
              <path fill="currentColor" d="M4 20c0-4.4 3.6-8 8-8s8 3.6 8 8v1H4z" />
            </svg>
          )}
        </div>
        <div className="my-page__profile-text">
          <h1>{user.nickname}</h1>
          <p className="my-page__stats">
            게시물 {user.postCount}개 · 받은 별 {user.starCount}개
            {user.githubLogin && (
              <>
                {' · '}
                <a href={`https://github.com/${user.githubLogin}`} target="_blank" rel="noreferrer">
                  GitHub @{user.githubLogin}
                </a>
              </>
            )}
          </p>
          {user.bio && <p className="my-page__bio">{user.bio}</p>}
        </div>
        <div className="my-page__actions">
          <button type="button" onClick={startEditing}>
            프로필 수정
          </button>
          <button type="button" onClick={startChangingPassword}>
            비밀번호 변경
          </button>
          <button type="button" onClick={handleLogout}>
            로그아웃
          </button>
        </div>
      </div>

      {editing && (
        <form className="profile-form" onSubmit={saveProfile}>
          <label className="profile-form__field">
            <span>닉네임</span>
            <input type="text" value={nickname} onChange={(e) => setNickname(e.target.value)} required />
          </label>
          <label className="profile-form__field">
            <span>아바타 URL (비우면 삭제)</span>
            <input
              type="url"
              value={avatarUrl}
              onChange={(e) => setAvatarUrl(e.target.value)}
              placeholder="https://..."
            />
          </label>
          <label className="profile-form__field">
            <span>소개 (비우면 삭제)</span>
            <textarea value={bio} onChange={(e) => setBio(e.target.value)} rows={3} maxLength={300} />
          </label>
          {error && <p className="profile-form__error">{error}</p>}
          <div className="profile-form__buttons">
            <button type="submit" disabled={saving}>
              저장
            </button>
            <button type="button" onClick={() => setEditing(false)}>
              취소
            </button>
          </div>
        </form>
      )}

      {!changingPassword && passwordChanged && (
        <p className="my-page__notice">비밀번호가 변경되었습니다.</p>
      )}

      {changingPassword && (
        <form className="profile-form" onSubmit={submitPasswordChange}>
          <label className="profile-form__field">
            <span>현재 비밀번호</span>
            <input
              type="password"
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
              autoComplete="current-password"
              required
            />
          </label>
          <label className="profile-form__field">
            <span>새 비밀번호</span>
            <input
              type="password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              autoComplete="new-password"
              placeholder="8자 이상, 특수문자 포함"
              required
            />
          </label>
          <ul className="password-rules" aria-live="polite">
            <li className={[...newPassword].length >= PASSWORD_MIN_LENGTH ? 'is-ok' : ''}>
              {PASSWORD_MIN_LENGTH}자 이상
            </li>
            <li className={hasSpecialChar(newPassword) ? 'is-ok' : ''}>
              특수문자(!@#$%^&* 등) 1개 이상
            </li>
            <li
              className={
                newPasswordConfirm.length > 0 && newPassword === newPasswordConfirm ? 'is-ok' : ''
              }
            >
              새 비밀번호 확인 일치
            </li>
          </ul>
          <label className="profile-form__field">
            <span>새 비밀번호 확인</span>
            <input
              type="password"
              value={newPasswordConfirm}
              onChange={(e) => setNewPasswordConfirm(e.target.value)}
              autoComplete="new-password"
              required
            />
          </label>
          {passwordError && <p className="profile-form__error">{passwordError}</p>}
          <div className="profile-form__buttons">
            <button type="submit" disabled={passwordSaving}>
              변경
            </button>
            <button type="button" onClick={() => setChangingPassword(false)}>
              취소
            </button>
          </div>
        </form>
      )}

      <h2 className="my-page__section-title">내가 작성한 게시물</h2>
      {!editing && error && <p className="my-page__empty">{error}</p>}
      {posts === null ? (
        <p className="my-page__empty">불러오는 중...</p>
      ) : posts.items.length === 0 ? (
        <p className="my-page__empty">아직 작성한 게시물이 없습니다.</p>
      ) : (
        <div className="my-page__list">
          {posts.items.map((post) => (
            <PostCard key={post.id} post={post} />
          ))}
        </div>
      )}
    </div>
  )
}

export default MyPage
