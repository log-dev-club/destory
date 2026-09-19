import { useEffect, useState } from 'react'
import { Link, useOutletContext, useParams } from 'react-router-dom'
import PostCard from '../components/PostCard'
import type { LayoutContext } from '../components/Layout'
import { fetchUserProfile } from '../api/users'
import { fetchUserPosts } from '../api/posts'
import { setUserRole } from '../api/admin'
import type { PostPage, UserProfile } from '../types'
import './UserProfilePage.css'

const ROLE_LABEL: Record<string, string> = {
  admin: '관리자',
  super_admin: '최고 관리자',
}

function UserProfilePage() {
  const { nickname } = useParams<{ nickname: string }>()
  const { user: viewer } = useOutletContext<LayoutContext>()

  const [profile, setProfile] = useState<UserProfile | null>(null)
  const [posts, setPosts] = useState<PostPage | null>(null)
  const [notFound, setNotFound] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [updatingRole, setUpdatingRole] = useState(false)

  useEffect(() => {
    if (!nickname) return
    let cancelled = false
    setProfile(null)
    setPosts(null)
    setNotFound(false)
    Promise.all([fetchUserProfile(nickname), fetchUserPosts(nickname)])
      .then(([loadedProfile, loadedPosts]) => {
        if (cancelled) return
        setProfile(loadedProfile)
        setPosts(loadedPosts)
      })
      .catch(() => {
        if (!cancelled) setNotFound(true)
      })
    return () => {
      cancelled = true
    }
  }, [nickname])

  const canManageRole =
    viewer?.role === 'super_admin' &&
    profile !== null &&
    profile.role !== 'super_admin' &&
    profile.nickname !== viewer.nickname

  const toggleAdmin = async () => {
    if (!profile) return
    setUpdatingRole(true)
    setError(null)
    try {
      const nextRole = profile.role === 'admin' ? 'user' : 'admin'
      const updated = await setUserRole(profile.nickname, nextRole)
      setProfile(updated)
    } catch (err) {
      setError(err instanceof Error ? err.message : '역할 변경에 실패했습니다')
    } finally {
      setUpdatingRole(false)
    }
  }

  if (notFound) {
    return (
      <div className="user-profile user-profile--empty">
        <p>사용자를 찾을 수 없습니다.</p>
        <Link to="/">목록으로 돌아가기</Link>
      </div>
    )
  }

  if (!profile) {
    return (
      <div className="user-profile user-profile--empty">
        <p>불러오는 중...</p>
      </div>
    )
  }

  return (
    <div className="user-profile">
      <div className="user-profile__header">
        <div className="user-profile__avatar">
          {profile.avatarUrl ? (
            <img src={profile.avatarUrl} alt="" width="72" height="72" style={{ borderRadius: '50%' }} />
          ) : (
            <svg viewBox="0 0 24 24" width="48" height="48" aria-hidden="true">
              <circle cx="12" cy="8" r="4" fill="currentColor" />
              <path fill="currentColor" d="M4 20c0-4.4 3.6-8 8-8s8 3.6 8 8v1H4z" />
            </svg>
          )}
        </div>
        <div className="user-profile__text">
          <h1>
            {profile.nickname}
            {ROLE_LABEL[profile.role] && (
              <span className={`user-profile__role user-profile__role--${profile.role}`}>
                {ROLE_LABEL[profile.role]}
              </span>
            )}
          </h1>
          <p className="user-profile__stats">
            게시물 {profile.postCount}개 · 받은 별 {profile.starCount}개
            {profile.githubLogin && (
              <>
                {' · '}
                <a href={`https://github.com/${profile.githubLogin}`} target="_blank" rel="noreferrer">
                  GitHub @{profile.githubLogin}
                </a>
              </>
            )}
          </p>
          {profile.bio && <p className="user-profile__bio">{profile.bio}</p>}
        </div>
        {canManageRole && (
          <div className="user-profile__actions">
            <button type="button" onClick={toggleAdmin} disabled={updatingRole}>
              {profile.role === 'admin' ? '관리자 권한 해제' : '관리자 권한 부여'}
            </button>
          </div>
        )}
      </div>

      {error && <p className="user-profile__error">{error}</p>}

      <h2 className="user-profile__section-title">작성한 게시물</h2>
      {posts === null ? (
        <p className="user-profile__empty">불러오는 중...</p>
      ) : posts.items.length === 0 ? (
        <p className="user-profile__empty">아직 작성한 게시물이 없습니다.</p>
      ) : (
        <div className="user-profile__list">
          {posts.items.map((post) => (
            <PostCard key={post.id} post={post} />
          ))}
        </div>
      )}
    </div>
  )
}

export default UserProfilePage
