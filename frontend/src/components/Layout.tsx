import { useState } from 'react'
import { Outlet } from 'react-router-dom'
import Header from './Header'

function Layout() {
  const [query, setQuery] = useState('')

  return (
    <div className="app-shell">
      <Header query={query} onQueryChange={setQuery} />
      <main className="app-content">
        <Outlet context={{ query }} />
      </main>
    </div>
  )
}

export default Layout
