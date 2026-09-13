import { useState } from 'react'

// 팀원 참고: 현재는 레이아웃 골격만 있는 상태입니다.
// 헤더(검색/필터/마이페이지)와 게시물 목록은 각 기능 담당자가 컴포넌트로 분리해 구현해 주세요.
function App() {
  const [query, setQuery] = useState('')

  return (
    <div className="layout">
      <header className="header">
        <span className="logo">destory</span>
        <input
          className="search"
          type="search"
          placeholder="검색"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <nav className="nav">
          <a href="#">새 글</a>
          <a href="#">마이페이지</a>
        </nav>
      </header>

      <main className="main">
        <ul className="post-list">
          {/* TODO: GET /api/posts 로 최신 게시물 목록 조회 */}
        </ul>
        <p className="empty">아직 게시물이 없습니다.</p>
      </main>
    </div>
  )
}

export default App
