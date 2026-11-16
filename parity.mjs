import assert from "node:assert"
import fs from "node:fs"
import os from "node:os"
import path from "node:path"
import { pathToFileURL } from "node:url"

import esbuild from "esbuild"

const here = path.dirname(new URL(import.meta.url).pathname)
const out = path.join(fs.mkdtempSync(path.join(os.tmpdir(), "highlight-")), "highlight.mjs")
await esbuild.build({ entryPoints: [path.join(here, "js/index.ts")], bundle: true, format: "esm", outfile: out, loader: { ".wasm": "binary" }, logLevel: "silent" })
const { tokenize, TOKEN_KINDS: KINDS } = await import(pathToFileURL(out).href)
const samples = JSON.parse(fs.readFileSync(path.join(here, "parity.json"), "utf8"))
let tokens = 0
for (const { language, code, tokens: flat } of samples) {
  const expected = []
  for (let i = 0, at = 0; i < flat.length; i += 2, at += flat[i - 1]) expected.push([KINDS[flat[i]], code.slice(at, at + flat[i + 1])])
  assert.deepStrictEqual(tokenize(code, language), expected, `${language}: ${code.slice(0, 40)}`)
  tokens += expected.length
}
console.log(`highlight parity ok: ${samples.length} samples, ${tokens} tokens`)
