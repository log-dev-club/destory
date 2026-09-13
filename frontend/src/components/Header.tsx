import { useEffect, useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import SearchBar from './SearchBar'
import type { UserProfile } from '../types'
import './Header.css'

interface HeaderProps {
  query: string
  onQueryChange: (value: string) => void
  user: UserProfile | null
}

function Header({ query, onQueryChange, user }: HeaderProps) {
  const [scrolled, setScrolled] = useState(false)
  const navigate = useNavigate()

  useEffect(() => {
    const handleScroll = () => setScrolled(window.scrollY > 8)
    handleScroll()
    window.addEventListener('scroll', handleScroll, { passive: true })
    return () => window.removeEventListener('scroll', handleScroll)
  }, [])

  return (
    <>
      <div className={`site-header-group ${scrolled ? 'is-scrolled' : ''}`}>
        <header className="site-header">
          <div className="site-header__side" />
          <Link className="site-header__logo" to="/">
            .log
          </Link>
          <div className="site-header__side site-header__side--right">
            {user ? (
              <button
                type="button"
                className="site-header__profile"
                aria-label="마이페이지로 이동"
                title={user.nickname}
                onClick={() => navigate('/me')}
              >
                {user.avatarUrl ? (
                  <img className="site-header__avatar" src={user.avatarUrl} alt="" />
                ) : (
                  <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
                    <circle cx="12" cy="8" r="4" fill="currentColor" />
                    <path fill="currentColor" d="M4 20c0-4.4 3.6-8 8-8s8 3.6 8 8v1H4z" />
                  </svg>
                )}
                <span className="site-header__nickname">{user.nickname}</span>
              </button>
            ) : (
              <Link className="site-header__login" to="/login">
                로그인
              </Link>
            )}
          </div>
        </header>
        <div className="site-header__search-band">
          <SearchBar query={query} onQueryChange={onQueryChange} compact={scrolled} />
        </div>
      </div>
      <div className="site-header-spacer" aria-hidden="true" />
    </>
  )
}

export default Header
