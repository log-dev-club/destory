import { forwardRef, useImperativeHandle, useState } from 'react'
import type { SuggestionKeyDownProps, SuggestionProps } from '@tiptap/suggestion'
import type { SlashCommandItem } from './slashCommandItems'
import './SlashCommandList.css'

export interface SlashCommandListRef {
  onKeyDown: (props: SuggestionKeyDownProps) => boolean
}

const SlashCommandList = forwardRef<SlashCommandListRef, SuggestionProps<SlashCommandItem>>((props, ref) => {
  const [items, setItems] = useState(props.items)
  const [selectedIndex, setSelectedIndex] = useState(0)

  if (props.items !== items) {
    setItems(props.items)
    setSelectedIndex(0)
  }

  const selectItem = (index: number) => {
    const item = props.items[index]
    if (item) props.command(item)
  }

  useImperativeHandle(ref, () => ({
    onKeyDown: ({ event }) => {
      if (props.items.length === 0) return false

      const isArrowUp = event.key === 'ArrowUp' || event.keyCode === 38
      const isArrowDown = event.key === 'ArrowDown' || event.keyCode === 40
      const isEnter = event.key === 'Enter' || event.keyCode === 13

      if (isArrowUp) {
        setSelectedIndex((prev) => (prev + props.items.length - 1) % props.items.length)
        return true
      }
      if (isArrowDown) {
        setSelectedIndex((prev) => (prev + 1) % props.items.length)
        return true
      }
      if (isEnter) {
        selectItem(selectedIndex)
        return true
      }
      return false
    },
  }))

  if (props.items.length === 0) {
    return (
      <div className="slash-menu">
        <div className="slash-menu__empty">일치하는 명령어가 없습니다</div>
      </div>
    )
  }

  return (
    <div className="slash-menu">
      {props.items.map((item, index) => (
        <button
          type="button"
          key={item.title}
          className={`slash-menu__item ${index === selectedIndex ? 'is-selected' : ''}`}
          onMouseEnter={() => setSelectedIndex(index)}
          onClick={() => selectItem(index)}
        >
          <span className="slash-menu__icon">{item.icon}</span>
          <span className="slash-menu__text">
            <span className="slash-menu__title">{item.title}</span>
            <span className="slash-menu__description">{item.description}</span>
          </span>
        </button>
      ))}
    </div>
  )
})

SlashCommandList.displayName = 'SlashCommandList'

export default SlashCommandList
