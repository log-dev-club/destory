import './SearchBar.css'

interface SearchBarProps {
  query: string
  onQueryChange: (value: string) => void
  compact?: boolean
}

function SearchBar({ query, onQueryChange, compact }: SearchBarProps) {
  return (
    <div className={`search-bar ${compact ? 'search-bar--compact' : ''}`}>
      <svg className="search-bar__icon" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
        <path
          fill="currentColor"
          d="M10.68 11.74a6 6 0 1 1 1.06-1.06l3.04 3.04a.75.75 0 1 1-1.06 1.06zM12 6.5a5.5 5.5 0 1 0-11 0 5.5 5.5 0 0 0 11 0"
        />
      </svg>
      <input
        className="search-bar__input"
        type="text"
        value={query}
        onChange={(event) => onQueryChange(event.target.value)}
        placeholder="tag:react user:yangmin"
        aria-label="검색어 (tag:태그명 user:닉네임 형식 지원)"
      />
    </div>
  )
}

export default SearchBar
