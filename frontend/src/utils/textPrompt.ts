// window.prompt() 는 브라우저/웹뷰에 따라 아예 표시되지 않거나(모바일 Chrome 등) 조용히
// null 을 반환하는 경우가 있어, 화면에 직접 입력 칸을 띄우는 방식으로 대체한다.
import './textPrompt.css'

export interface TextPromptOptions {
  title: string
  placeholder?: string
  initialValue?: string
  confirmLabel?: string
}

export function promptText({
  title,
  placeholder = '',
  initialValue = '',
  confirmLabel = '삽입',
}: TextPromptOptions): Promise<string | null> {
  return new Promise((resolve) => {
    const overlay = document.createElement('div')
    overlay.className = 'text-prompt-overlay'

    const box = document.createElement('form')
    box.className = 'text-prompt'

    const titleEl = document.createElement('div')
    titleEl.className = 'text-prompt__title'
    titleEl.textContent = title
    box.appendChild(titleEl)

    const input = document.createElement('input')
    input.type = 'text'
    input.className = 'text-prompt__input'
    input.placeholder = placeholder
    input.value = initialValue
    box.appendChild(input)

    const actions = document.createElement('div')
    actions.className = 'text-prompt__actions'

    const cancelButton = document.createElement('button')
    cancelButton.type = 'button'
    cancelButton.className = 'text-prompt__cancel'
    cancelButton.textContent = '취소'
    actions.appendChild(cancelButton)

    const confirmButton = document.createElement('button')
    confirmButton.type = 'submit'
    confirmButton.className = 'text-prompt__confirm'
    confirmButton.textContent = confirmLabel
    actions.appendChild(confirmButton)

    box.appendChild(actions)
    overlay.appendChild(box)
    document.body.appendChild(overlay)

    const finish = (result: string | null) => {
      document.removeEventListener('keydown', onKeyDown)
      overlay.remove()
      resolve(result)
    }

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') finish(null)
    }

    box.addEventListener('submit', (event) => {
      event.preventDefault()
      const value = input.value.trim()
      finish(value || null)
    })
    cancelButton.addEventListener('click', () => finish(null))
    overlay.addEventListener('mousedown', (event) => {
      if (event.target === overlay) finish(null)
    })
    document.addEventListener('keydown', onKeyDown)

    requestAnimationFrame(() => {
      input.focus()
      input.select()
    })
  })
}
