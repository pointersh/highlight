#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Kind {
    Text = 0,
    Comment = 1,
    String = 2,
    Number = 3,
    Keyword = 4,
    Fn = 5,
    Add = 6,
    Del = 7,
    Meta = 8,
    Tag = 9,
}

impl Kind {
    const ALL: [Kind; 10] = [
        Kind::Text,
        Kind::Comment,
        Kind::String,
        Kind::Number,
        Kind::Keyword,
        Kind::Fn,
        Kind::Add,
        Kind::Del,
        Kind::Meta,
        Kind::Tag,
    ];

    fn name(self) -> &'static str {
        match self {
            Kind::Text => "text",
            Kind::Comment => "comment",
            Kind::String => "string",
            Kind::Number => "number",
            Kind::Keyword => "keyword",
            Kind::Fn => "fn",
            Kind::Add => "add",
            Kind::Del => "del",
            Kind::Meta => "meta",
            Kind::Tag => "tag",
        }
    }
}

static mut KIND_NAMES: Vec<u8> = Vec::new();

#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    C,
    Hash,
    Diff,
    Markup,
    Plain,
}

pub type Token = (Kind, usize, usize);

fn family(language: &[u16]) -> Family {
    let lower: Vec<u8> = language.iter().map(|&c| if c < 128 { (c as u8).to_ascii_lowercase() } else { 0 }).collect();
    match lower.as_slice() {
        b"py" | b"python" | b"sh" | b"bash" | b"shell" | b"zsh" | b"yaml" | b"yml" | b"toml" | b"rb" | b"ruby"
        | b"dockerfile" | b"makefile" | b"r" | b"ini" => Family::Hash,
        b"diff" | b"patch" => Family::Diff,
        b"html" | b"xml" | b"svg" | b"vue" => Family::Markup,
        b"md" | b"markdown" | b"text" | b"txt" => Family::Plain,
        _ => Family::C,
    }
}

fn is_keyword(word: &[u8]) -> bool {
    matches!(
        word,
        b"abstract" | b"as" | b"async" | b"await" | b"break" | b"case" | b"catch" | b"class" | b"const"
            | b"continue" | b"def" | b"default" | b"defer" | b"del" | b"delete" | b"do" | b"elif" | b"else"
            | b"enum" | b"except" | b"export" | b"extends" | b"false" | b"final" | b"finally" | b"fn"
            | b"for" | b"from" | b"func" | b"function" | b"go" | b"if" | b"impl" | b"implements"
            | b"import" | b"in" | b"instanceof" | b"interface" | b"is" | b"lambda" | b"let" | b"loop"
            | b"match" | b"mod" | b"module" | b"mut" | b"new" | b"nil" | b"none" | b"not" | b"null"
            | b"of" | b"or" | b"and" | b"package" | b"pass" | b"private" | b"protected" | b"pub"
            | b"public" | b"raise" | b"readonly" | b"return" | b"self" | b"static" | b"struct"
            | b"super" | b"switch" | b"this" | b"throw" | b"trait" | b"true" | b"try" | b"type"
            | b"typeof" | b"use" | b"var" | b"void" | b"where" | b"while" | b"with" | b"yield"
            | b"True" | b"False" | b"None" | b"unsafe" | b"crate" | b"ref" | b"move" | b"dyn"
            | b"local" | b"then" | b"end" | b"elseif"
    )
}

fn is_digit(c: u16) -> bool {
    (b'0' as u16..=b'9' as u16).contains(&c)
}

fn is_alpha(c: u16) -> bool {
    (b'a' as u16..=b'z' as u16).contains(&c) || (b'A' as u16..=b'Z' as u16).contains(&c)
}

fn is_word(c: u16) -> bool {
    is_alpha(c) || is_digit(c) || c == b'_' as u16
}

fn is_hex(c: u16) -> bool {
    is_digit(c) || (b'a' as u16..=b'f' as u16).contains(&c) || (b'A' as u16..=b'F' as u16).contains(&c)
}

fn is_line_terminator(c: u16) -> bool {
    c == 0x0a || c == 0x0d || c == 0x2028 || c == 0x2029
}

const NL: u16 = b'\n' as u16;

fn at(s: &[u16], i: usize, c: u8) -> bool {
    s.get(i) == Some(&(c as u16))
}

fn starts(s: &[u16], i: usize, lit: &[u8]) -> bool {
    s.len() >= i + lit.len() && lit.iter().enumerate().all(|(k, &c)| s[i + k] == c as u16)
}

fn find(s: &[u16], from: usize, lit: &[u8]) -> Option<usize> {
    (from..s.len()).find(|&i| starts(s, i, lit))
}

fn run(s: &[u16], mut i: usize, f: impl Fn(u16) -> bool) -> usize {
    while i < s.len() && f(s[i]) {
        i += 1;
    }
    i
}

fn boundary(s: &[u16], i: usize) -> bool {
    let before = i > 0 && is_word(s[i - 1]);
    let after = i < s.len() && is_word(s[i]);
    before != after
}

fn line_comment(s: &[u16], p: usize, lead: &[u8]) -> Option<usize> {
    starts(s, p, lead).then(|| run(s, p + lead.len(), |c| c != NL))
}

fn block(s: &[u16], p: usize, open: &[u8], close: &[u8]) -> Option<usize> {
    if !starts(s, p, open) {
        return None;
    }
    find(s, p + open.len(), close).map(|e| e + close.len())
}

fn quoted(s: &[u16], p: usize) -> Option<usize> {
    let q = *s.get(p)?;
    let multiline = q == b'`' as u16;
    if q != b'"' as u16 && q != b'\'' as u16 && !multiline {
        return None;
    }
    let mut i = p + 1;
    while i < s.len() {
        let c = s[i];
        if c == b'\\' as u16 {
            if i + 1 < s.len() && !is_line_terminator(s[i + 1]) {
                i += 2;
                continue;
            }
            return None;
        }
        if c == q {
            return Some(i + 1);
        }
        if c == NL && !multiline {
            return None;
        }
        i += 1;
    }
    None
}

fn number(s: &[u16], p: usize) -> Option<usize> {
    if !boundary(s, p) {
        return None;
    }
    let end_ok = |e: usize| boundary(s, e).then_some(e);
    if at(s, p, b'0') && at(s, p + 1, b'x') {
        let e = run(s, p + 2, |c| is_hex(c) || c == b'_' as u16);
        if e > p + 2 {
            if let Some(e) = end_ok(e) {
                return Some(e);
            }
        }
    }
    if !s.get(p).is_some_and(|&c| is_digit(c)) {
        return None;
    }
    let int = run(s, p + 1, |c| is_digit(c) || c == b'_' as u16);
    let frac = if at(s, int, b'.') && s.get(int + 1).is_some_and(|&c| is_digit(c)) {
        run(s, int + 1, is_digit)
    } else {
        int
    };
    let mut exp = None;
    if at(s, frac, b'e') {
        let d = if at(s, frac + 1, b'+') || at(s, frac + 1, b'-') { frac + 2 } else { frac + 1 };
        let e = run(s, d, is_digit);
        if e > d {
            exp = Some(e);
        }
    }
    exp.and_then(end_ok).or_else(|| end_ok(frac)).or_else(|| if frac > int { end_ok(int) } else { None })
}

fn word(s: &[u16], p: usize) -> Option<usize> {
    let c = *s.get(p)?;
    (is_alpha(c) || c == b'_' as u16 || c == b'$' as u16).then(|| run(s, p + 1, |c| is_word(c) || c == b'$' as u16))
}

fn scan(s: &[u16], out: &mut Vec<Token>, mut matcher: impl FnMut(&[u16], usize) -> Option<(Kind, usize)>) {
    let (mut last, mut p) = (0, 0);
    while p < s.len() {
        match matcher(s, p) {
            Some((kind, end)) => {
                if p > last {
                    out.push((Kind::Text, last, p - last));
                }
                out.push((kind, p, end - p));
                last = end;
                p = end;
            }
            None => p += 1,
        }
    }
    if last < s.len() {
        out.push((Kind::Text, last, s.len() - last));
    }
}

fn code_token(s: &[u16], p: usize, hash: bool) -> Option<(Kind, usize)> {
    let comment = if hash {
        line_comment(s, p, b"#")
    } else {
        line_comment(s, p, b"//").or_else(|| block(s, p, b"/*", b"*/"))
    };
    if let Some(e) = comment {
        return Some((Kind::Comment, e));
    }
    let triple = if hash {
        block(s, p, b"\"\"\"", b"\"\"\"").or_else(|| block(s, p, b"'''", b"'''"))
    } else {
        None
    };
    if let Some(e) = triple.or_else(|| quoted(s, p)) {
        return Some((Kind::String, e));
    }
    if let Some(e) = number(s, p) {
        return Some((Kind::Number, e));
    }
    let e = word(s, p)?;
    let text: Vec<u8> = s[p..e].iter().map(|&c| c as u8).collect();
    let kind = if is_keyword(&text) {
        Kind::Keyword
    } else if at(s, e, b'(') {
        Kind::Fn
    } else {
        Kind::Text
    };
    Some((kind, e))
}

fn markup_token(s: &[u16], p: usize) -> Option<(Kind, usize)> {
    if let Some(e) = block(s, p, b"<!--", b"-->") {
        return Some((Kind::Comment, e));
    }
    for q in [b"\"", b"'"] {
        if starts(s, p, q) {
            if let Some(e) = find(s, p + 1, q) {
                return Some((Kind::String, e + 1));
            }
        }
    }
    let name = |c: u16| is_word(c) || c == b':' as u16 || c == b'-' as u16;
    if at(s, p, b'<') {
        let from = if at(s, p + 1, b'/') { p + 2 } else { p + 1 };
        let e = run(s, from, name);
        if e > from {
            return Some((Kind::Tag, e));
        }
    }
    if at(s, p, b'>') {
        return Some((Kind::Tag, p + 1));
    }
    if at(s, p, b'/') && at(s, p + 1, b'>') {
        return Some((Kind::Tag, p + 2));
    }
    None
}

fn diff(s: &[u16], out: &mut Vec<Token>) {
    let mut start = 0;
    loop {
        let end = s[start..].iter().position(|&c| c == NL).map_or(s.len(), |i| start + i + 1);
        let line = &s[start..end];
        let kind = if starts(line, 0, b"@@") {
            Kind::Meta
        } else if starts(line, 0, b"+") && !starts(line, 0, b"+++") {
            Kind::Add
        } else if starts(line, 0, b"-") && !starts(line, 0, b"---") {
            Kind::Del
        } else {
            Kind::Text
        };
        out.push((kind, start, end - start));
        if end >= s.len() {
            break;
        }
        start = end;
    }
}

pub fn tokenize(code: &[u16], language: &[u16]) -> Vec<Token> {
    let mut out = Vec::new();
    match family(language) {
        Family::Diff => diff(code, &mut out),
        Family::Markup => scan(code, &mut out, markup_token),
        Family::Plain => out.push((Kind::Text, 0, code.len())),
        f => scan(code, &mut out, |s, p| code_token(s, p, f == Family::Hash)),
    }
    out
}

static mut OUTPUT: Vec<u32> = Vec::new();

#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u16 {
    let mut buf = Vec::<u16>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u16, len: usize) {
    drop(Vec::from_raw_parts(ptr, 0, len));
}

#[no_mangle]
pub unsafe extern "C" fn highlight(ptr: *const u16, code_len: usize, lang_len: usize) -> *const u32 {
    let input = std::slice::from_raw_parts(ptr, code_len + lang_len);
    let output = &mut *std::ptr::addr_of_mut!(OUTPUT);
    output.clear();
    output.push(0);
    for (kind, start, len) in tokenize(&input[..code_len], &input[code_len..]) {
        output.extend([start as u32, len as u32, kind as u32]);
    }
    output[0] = ((output.len() - 1) / 3) as u32;
    output.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn kind_names() -> *const u8 {
    let names = &mut *std::ptr::addr_of_mut!(KIND_NAMES);
    if names.is_empty() {
        let joined = Kind::ALL.map(Kind::name).join(" ");
        names.extend((joined.len() as u32).to_le_bytes());
        names.extend(joined.bytes());
    }
    names.as_ptr()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_are_numbered_in_order() {
        for (i, kind) in Kind::ALL.into_iter().enumerate() {
            assert_eq!(kind as usize, i, "{}", kind.name());
        }
    }

    fn tokens(code: &str, language: &str) -> Vec<(Kind, String)> {
        let units: Vec<u16> = code.encode_utf16().collect();
        tokenize(&units, &language.encode_utf16().collect::<Vec<_>>())
            .into_iter()
            .map(|(k, s, l)| (k, String::from_utf16(&units[s..s + l]).unwrap()))
            .collect()
    }

    fn t(list: &[(Kind, &str)]) -> Vec<(Kind, String)> {
        list.iter().map(|&(k, s)| (k, s.to_string())).collect()
    }

    use Kind::{Add, Comment, Del, Fn, Keyword, Meta, Number, Tag, Text};

    #[test]
    fn typescript() {
        assert_eq!(
            tokens("const x = foo(0x1F, 1.5e3) // hi", "ts"),
            t(&[
                (Keyword, "const"), (Text, " "), (Text, "x"), (Text, " = "), (Fn, "foo"), (Text, "("),
                (Number, "0x1F"), (Text, ", "), (Number, "1.5e3"), (Text, ") "), (Comment, "// hi"),
            ])
        );
    }

    #[test]
    fn multiline_block_comment_and_template() {
        assert_eq!(
            tokens("/* a\nb */`x\n${y}`'no\n'", "js"),
            t(&[(Comment, "/* a\nb */"), (Kind::String, "`x\n${y}`"), (Text, "'"), (Text, "no"), (Text, "\n'")])
        );
    }

    #[test]
    fn python_triple_quotes() {
        assert_eq!(
            tokens("def f():\n    \"\"\"doc\nmore\"\"\" # c\n    return None", "python"),
            t(&[
                (Keyword, "def"), (Text, " "), (Fn, "f"), (Text, "():\n    "), (Kind::String, "\"\"\"doc\nmore\"\"\""),
                (Text, " "), (Comment, "# c"), (Text, "\n    "), (Keyword, "return"), (Text, " "), (Keyword, "None"),
            ])
        );
    }

    #[test]
    fn unclosed_triple_falls_back_to_empty_string() {
        assert_eq!(tokens("\"\"\"x", "py"), t(&[(Kind::String, "\"\""), (Text, "\""), (Text, "x")]));
    }

    #[test]
    fn number_backtracking() {
        assert_eq!(tokens("1.5x 1e5x 0xg", "c"), t(&[
            (Number, "1"), (Text, ".5"), (Text, "x"), (Text, " 1"), (Text, "e5x"), (Text, " 0"), (Text, "xg"),
        ]));
    }

    #[test]
    fn unicode() {
        assert_eq!(
            tokens("naïve(\"héllo 🎉\") // ✓", "ts"),
            t(&[(Text, "na"), (Text, "ï"), (Fn, "ve"), (Text, "("), (Kind::String, "\"héllo 🎉\""), (Text, ") "), (Comment, "// ✓")])
        );
    }

    #[test]
    fn markup() {
        assert_eq!(
            tokens("<!-- a\n--><a href=\"x\"></a><br/>", "HTML"),
            t(&[
                (Comment, "<!-- a\n-->"), (Tag, "<a"), (Text, " href="), (Kind::String, "\"x\""), (Tag, ">"), (Tag, "</a"),
                (Tag, ">"), (Tag, "<br"), (Tag, "/>"),
            ])
        );
    }

    #[test]
    fn diff_and_plain() {
        assert_eq!(
            tokens("@@ -1 +1 @@\n-a\n+b\n+++ c\n", "diff"),
            t(&[(Meta, "@@ -1 +1 @@\n"), (Del, "-a\n"), (Add, "+b\n"), (Text, "+++ c\n")])
        );
        assert_eq!(tokens("", "diff"), t(&[(Text, "")]));
        assert_eq!(tokens("", "text"), t(&[(Text, "")]));
        assert_eq!(tokens("", "ts"), t(&[]));
    }
}
