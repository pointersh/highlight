import { Token, tokenize } from "./index"

export * from "./index"

const render = (tokens: readonly Token[]) =>
  tokens.map(([kind, text], i) => (kind === "text" ? text : <span key={i} className={`hl-${kind}`}>{text}</span>))

interface CodeProps {
  code: string
  language?: string
  startLine?: number
  className?: string
}

export const Code = ({ code, language, startLine, className }: CodeProps) => {
  if (startLine === undefined) {
    return (
      <pre className={["hl", className].filter(Boolean).join(" ")}>
        <code>{render(tokenize(code, language))}</code>
      </pre>
    )
  }
  return (
    <pre className={["hl", "hl-numbered", className].filter(Boolean).join(" ")}>
      <code>
        {code.split("\n").map((line, i) => (
          <span key={i} className="hl-line" data-line={startLine + i}>
            {render(tokenize(line, language))}
            {"\n"}
          </span>
        ))}
      </code>
    </pre>
  )
}
