# @pointersh/highlight

A small syntax highlighter: a dependency-free Rust tokenizer compiled to
WebAssembly (about 30 KB), with a typed TypeScript and React API.

- **Typed end to end.** Token kinds are defined once, in Rust. The module
  exports their names; on load the JS side checks them against
  `TOKEN_KINDS`, and refuses to start if they differ. `TokenKind` is derived
  from that list, so a kind added in Rust and not in TypeScript fails loudly
  instead of rendering wrong.
- **UTF-16 offsets.** The tokenizer works on JavaScript string code units, so
  token boundaries are string indices with no conversion, emoji included.
- **Theming through CSS variables** (`--hl-keyword`, `--hl-string`, …).
  Inside a VS Code webview the defaults follow the editor theme.
- **Diff-aware:** `+`, `-` and `@@` lines get their own kinds.

```ts
import { highlight, tokenize, type Token } from "@pointersh/highlight"
import { Code } from "@pointersh/highlight/react"
import "@pointersh/highlight/theme.css"

const tokens: Token[] = tokenize(src, "python")
el.innerHTML = `<pre class="hl"><code>${highlight(src, "python")}</code></pre>`
const view = <Code code={src} language="python" startLine={1} />
```

Token kinds: `text`, `comment`, `string`, `number`, `keyword`, `fn`, `add`,
`del`, `meta`, `tag`.

## Install

```sh
git clone https://github.com/pointersh/highlight.git && cd highlight
rustup target add wasm32-unknown-unknown
npm run build
```

Your bundler needs a binary loader for `.wasm` (esbuild:
`loader: { ".wasm": "binary" }`). Building, checks and the WebAssembly
interface: [docs/internals.md](docs/internals.md).

Part of [Pointer](https://github.com/pointersh/Pointer). MIT.
