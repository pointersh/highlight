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

## Build and check

```sh
rustup target add wasm32-unknown-unknown
npm run build    # cargo build --release --target wasm32-unknown-unknown, copied to highlight.wasm
npm run check    # parity against the saved output of the original TypeScript tokenizer
cargo test
```

The package ships TypeScript source and imports `highlight.wasm` as bytes, so
the bundler needs a binary loader for `.wasm` (esbuild:
`loader: { ".wasm": "binary" }`). Inside a webview, the Content Security
Policy needs `'wasm-unsafe-eval'` in `script-src`.

## WebAssembly interface

| Export | Signature | Purpose |
|---|---|---|
| `alloc` | `(units) → ptr` | Space for the code followed by the language name, as UTF-16 |
| `highlight` | `(ptr, codeUnits, langUnits) → ptr` | Tokenizes; the result is `[count, (start, length, kind) × count]` as `u32` |
| `dealloc` | `(ptr, units)` | Frees the input |
| `kind_names` | `() → ptr` | A `u32` byte length, then the kind names in kind order, space-separated |

Part of [Pointer](https://github.com/pointersh/Pointer). MIT.
