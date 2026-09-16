// 이모지 대신 쓰는 심플한 라인 아이콘 모음. 색상은 currentColor 를 사용해 부모 텍스트 색을 따라간다.

interface IconProps {
  className?: string
}

export function CommentIcon({ className }: IconProps) {
  return (
    <svg
      className={className}
      width="14"
      height="14"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.4"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M2 3.5h12v7H6.5L3.5 13v-2.5H2v-7Z" />
    </svg>
  )
}

export function ImageIcon({ className }: IconProps) {
  return (
    <svg
      className={className}
      width="14"
      height="14"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.4"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <rect x="2" y="3" width="12" height="10" rx="1.5" />
      <circle cx="5.5" cy="6.5" r="1.1" fill="currentColor" stroke="none" />
      <path d="M14 10.5 10.5 7 5 12.5" />
    </svg>
  )
}
