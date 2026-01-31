/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'

  const component: DefineComponent<{}, {}, any>
  export default component
}

declare module 'subsrt' {
  interface ConvertOptions {
    format?: 'srt' | 'vtt' | 'lrc' | 'sbv' | 'sub' | 'json'
    fps?: number
  }

  function convert(content: string, options?: ConvertOptions): string
  function parse(content: string): any[]
  function build(captions: any[], options?: ConvertOptions): string
  function resync(content: string, time: number): string
  function list(): string[]

  export { build, convert, list, parse, resync }
  export default { convert, parse, build, resync, list }
}
