import { Extension } from '@tiptap/core'
import Suggestion from '@tiptap/suggestion'
import type { SuggestionOptions } from '@tiptap/suggestion'
import { ReactRenderer } from '@tiptap/react'
import SlashCommandList from './SlashCommandList'
import type { SlashCommandListRef } from './SlashCommandList'
import { getSlashCommandItems } from './slashCommandItems'
import type { SlashCommandItem } from './slashCommandItems'

export interface SlashCommandOptions {
  suggestion: Partial<SuggestionOptions<SlashCommandItem>>
}

const SlashCommand = Extension.create<SlashCommandOptions>({
  name: 'slashCommand',

  addOptions() {
    return {
      suggestion: {
        char: '/',
        allowSpaces: false,
        startOfLine: false,
        command: ({ editor, range, props }) => {
          props.command({ editor, range })
        },
      },
    }
  },

  addProseMirrorPlugins() {
    return [
      Suggestion({
        editor: this.editor,
        ...this.options.suggestion,
        items: ({ query }) => getSlashCommandItems(query),
        render: () => {
          let component: ReactRenderer<SlashCommandListRef> | null = null
          let unmount: (() => void) | null = null

          return {
            onStart: (props) => {
              component = new ReactRenderer(SlashCommandList, {
                props,
                editor: props.editor,
              })
              unmount = props.mount(component.element as HTMLElement)
            },
            onUpdate: (props) => {
              component?.updateProps(props)
            },
            onKeyDown: (props) => {
              if (props.event.key === 'Escape' || props.event.keyCode === 27) {
                unmount?.()
                return true
              }
              return component?.ref?.onKeyDown(props) ?? false
            },
            onExit: () => {
              unmount?.()
              component?.destroy()
            },
          }
        },
      }),
    ]
  },
})

export default SlashCommand
