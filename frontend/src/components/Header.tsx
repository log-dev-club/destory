import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import SearchBar from './SearchBar'
import './Header.css'

interface HeaderProps {
  query: string
  onQueryChange: (value: string) => void
}

function Header({ query, onQueryChange }: HeaderProps) {
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
          <a className="site-header__logo" href="/">
            .log
          </a>
          <div className="site-header__side site-header__side--right">
            <button
              type="button"
              className="site-header__profile"
              aria-label="마이페이지로 이동"
              onClick={() => navigate('/me')}
            >
              <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
                <circle cx="12" cy="8" r="4" fill="currentColor" />
                <path fill="currentColor" d="M4 20c0-4.4 3.6-8 8-8s8 3.6 8 8v1H4z" />
              </svg>
            </button>
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
