import bytes from "../highlight.wasm"

export const TOKEN_KINDS = ["text", "comment", "string", "number", "keyword", "fn", "add", "del", "meta", "tag"] as const

export type TokenKind = (typeof TOKEN_KINDS)[number]
export type Token = readonly [kind: TokenKind, text: string]

interface Exports {
  memory: WebAssembly.Memory
  alloc(units: number): number
  dealloc(ptr: number, units: number): void
  highlight(ptr: number, codeUnits: number, langUnits: number): number
  kind_names(): number
}

const FUNCTIONS = ["alloc", "dealloc", "highlight", "kind_names"] as const

const isExports = (value: WebAssembly.Exports): value is WebAssembly.Exports & Exports =>
  value.memory instanceof WebAssembly.Memory && FUNCTIONS.every((name) => typeof value[name] === "function")

const load = (): Exports => {
  const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes))
  if (!isExports(exports)) throw new Error(`highlight.wasm must export memory and ${FUNCTIONS.join(", ")}`)
  const at = exports.kind_names()
  const length = new DataView(exports.memory.buffer).getUint32(at, true)
  const names = new TextDecoder().decode(new Uint8Array(exports.memory.buffer, at + 4, length)).split(" ")
  if (names.join(" ") !== TOKEN_KINDS.join(" ")) {
    throw new Error(`highlight.wasm token kinds [${names}] do not match TOKEN_KINDS [${TOKEN_KINDS}]`)
  }
  return exports
}

const wasm = load()

export const tokenize = (code: string, language = "text"): Token[] => {
  const lang = language.toLowerCase()
  const units = code.length + lang.length
  const ptr = wasm.alloc(units)
  const input = new Uint16Array(wasm.memory.buffer, ptr, units)
  for (let i = 0; i < code.length; i++) input[i] = code.charCodeAt(i)
  for (let i = 0; i < lang.length; i++) input[code.length + i] = lang.charCodeAt(i)
  const at = wasm.highlight(ptr, code.length, lang.length)
  const count = new Uint32Array(wasm.memory.buffer, at, 1)[0]
  const words = new Uint32Array(wasm.memory.buffer, at + 4, count * 3)
  const tokens: Token[] = []
  for (let i = 0; i < words.length; i += 3) {
    tokens.push([TOKEN_KINDS[words[i + 2]], code.slice(words[i], words[i] + words[i + 1])])
  }
  wasm.dealloc(ptr, units)
  return tokens
}

const ESCAPES: Record<string, string> = { "&": "&amp;", "<": "&lt;", ">": "&gt;", "\"": "&quot;", "'": "&#39;" }
const escape = (text: string) => text.replace(/[&<>"']/g, (c) => ESCAPES[c])

export const toHtml = (tokens: readonly Token[]): string =>
  tokens.map(([kind, text]) => (kind === "text" ? escape(text) : `<span class="hl-${kind}">${escape(text)}</span>`)).join("")

export const highlight = (code: string, language?: string): string => toHtml(tokenize(code, language))
