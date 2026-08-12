//! Heuristic source-symbol extraction (docs/10 §2, docs/21 Stage 6).
//!
//! Before a real Tree-sitter adapter lands (ADR-011 Proposed), this provides
//! deterministic, line-based symbol extraction for the common languages:
//! functions, structs, classes, traits/interfaces, enums and constants. It is
//! explicitly heuristic — the same labeling discipline as the manifest and
//! commit-classification parsers — and reads only Git trees (never the
//! worktree).

use gitx_git::Repository;
use std::path::{Path, PathBuf};

/// A single extracted symbol.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Symbol {
    pub name: String,
    /// Function | Method | Struct | Class | Trait | Interface | Enum | Const
    pub kind: String,
    /// 1-based source line.
    pub line: u32,
}

/// Language for a path's extension (lowercase, no dot), or `None` for
/// unsupported languages. Public so the pipeline and CLI can label
/// complexity sources (docs/10 §2: never silently zero a missing input).
pub fn lang_of(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    match ext.as_str() {
        "rs" => Some("rust"),
        "py" => Some("python"),
        "go" => Some("go"),
        "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" => Some("js"),
        "c" | "h" | "cc" | "cpp" | "hpp" | "cxx" => Some("cpp"),
        "java" | "kt" => Some("java"),
        "rb" => Some("ruby"),
        "php" => Some("php"),
        _ => None,
    }
}

/// Extract symbols from every supported source file in `tree_id` (bounded:
/// reads only the tree blobs, deterministic order). Returns `(path, symbols)`
/// sorted by path.
pub fn extract_symbols_from_tree(
    repo: &Repository,
    tree_id: gitx_git::models::ObjectId,
) -> anyhow::Result<Vec<(PathBuf, Vec<Symbol>)>> {
    let mut out = Vec::new();
    for path in repo.list_blobs(tree_id)? {
        if lang_of(&path).is_none() {
            continue;
        }
        let Ok(Some(bytes)) = repo.blob_at_path(tree_id, &path) else {
            continue;
        };
        let content = String::from_utf8_lossy(&bytes);
        let symbols = extract_symbols(&content, lang_of(&path).unwrap_or(""));
        if !symbols.is_empty() {
            out.push((path, symbols));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

/// Line-based symbol extraction for one file's content.
pub fn extract_symbols(content: &str, lang: &str) -> Vec<Symbol> {
    let mut out = Vec::new();
    for (idx, raw) in content.lines().enumerate() {
        let line_no = (idx + 1) as u32;
        let line = raw.trim();
        if line.is_empty() || is_comment(line, lang) {
            continue;
        }
        for (name, kind) in match_symbol(line, lang) {
            out.push(Symbol {
                name,
                kind: kind.to_string(),
                line: line_no,
            });
        }
    }
    out
}

/// Heuristic function/method count for `content` (docs/10 §2 complexity
/// signal). Returns `0` for languages without an extractor; callers keep
/// LOC as the always-available fallback and label the source.
pub fn function_count(content: &str, lang: &str) -> u32 {
    extract_symbols(content, lang)
        .into_iter()
        .filter(|s| matches!(s.kind.as_str(), "Function" | "Method"))
        .count() as u32
}

fn is_comment(line: &str, lang: &str) -> bool {
    if line.starts_with("//") || line.starts_with("/*") || line.starts_with('*') {
        return true;
    }
    if lang == "python" || lang == "ruby" {
        return line.starts_with('#');
    }
    if lang == "php" {
        return line.starts_with('#') || line.starts_with("//") || line.starts_with("/*");
    }
    false
}

/// Match one line against the language's declaration shapes. Returns
/// `(name, kind)` pairs (a line can declare at most one symbol; kept as a
/// small Vec for uniformity).
fn match_symbol(line: &str, lang: &str) -> Vec<(String, &'static str)> {
    let mut out = Vec::new();
    let (before, rest) = match line.split_once('(') {
        Some((b, r)) if !r.starts_with(')') => (b, Some(r)),
        Some((b, _)) => (b, None), // `fn f()` — still a declaration
        None => (line, None),
    };
    let _ = rest;

    match lang {
        "rust" => {
            // Strip visibility/async/unsafe/extern modifiers so `pub fn`,
            // `pub(crate) fn`, `async fn`, `unsafe fn` all match on the
            // declaration keyword.
            let before = strip_rust_modifiers(before);
            if let Some(name) = after_keyword(before, "fn") {
                out.push((name, "Function"));
            } else if let Some(name) = after_keyword(before, "struct") {
                out.push((name, "Struct"));
            } else if let Some(name) = after_keyword(before, "enum") {
                out.push((name, "Enum"));
            } else if let Some(name) = after_keyword(before, "trait") {
                out.push((name, "Trait"));
            } else if let Some(name) = after_keyword(before, "type") {
                out.push((name, "Type"));
            } else if let Some(name) = after_keyword(before, "const") {
                out.push((name, "Const"));
            } else if let Some(name) = after_keyword(before, "static") {
                out.push((name, "Const"));
            }
        }
        "python" => {
            let stripped = line
                .strip_prefix("async ")
                .or_else(|| line.strip_prefix("async\t"))
                .unwrap_or(line);
            if let Some(name) = after_keyword(stripped, "def") {
                out.push((name, "Function"));
            } else if let Some(name) = after_keyword(stripped, "class") {
                out.push((name, "Class"));
            }
        }
        "go" => {
            // Use the full line: receiver methods (`func (r T) Name(`) must be
            // handled before splitting at the first `(`.
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("func ") {
                let is_method = rest.starts_with('(');
                let name = if is_method {
                    rest.split_once(')')
                        .map(|(_, after)| {
                            after
                                .split_whitespace()
                                .next()
                                .unwrap_or("")
                                .split('(')
                                .next()
                                .unwrap_or("")
                                .to_string()
                        })
                        .unwrap_or_default()
                } else {
                    rest.split('(').next().unwrap_or("").trim().to_string()
                };
                if !name.is_empty() && name != "func" {
                    out.push((
                        name.to_string(),
                        if is_method { "Method" } else { "Function" },
                    ));
                }
            } else if let Some(rest) = trimmed.strip_prefix("type ") {
                let name = first_word(rest).unwrap_or("");
                let kind = if rest.contains("interface") {
                    "Interface"
                } else if rest.contains("struct") {
                    "Struct"
                } else {
                    "Type"
                };
                if !name.is_empty() {
                    out.push((name.to_string(), kind));
                }
            } else if let Some(name) = after_keyword(trimmed, "const") {
                out.push((name, "Const"));
            }
        }
        "js" => {
            if let Some(name) = after_keyword(before, "function") {
                out.push((name, "Function"));
            } else if let Some(name) = after_keyword(before, "class") {
                out.push((name, "Class"));
            } else if let Some(name) = after_keyword(before, "interface") {
                out.push((name, "Interface"));
            } else if let Some(name) = after_keyword(before, "enum") {
                out.push((name, "Enum"));
            } else if let Some(name) = after_keyword(before, "type") {
                out.push((name, "Type"));
            }
        }
        "cpp" => {
            // `Type name(` / `static Type name(` — skip control keywords.
            let trimmed = before.trim_start();
            let rest = trimmed
                .strip_prefix("static ")
                .or_else(|| trimmed.strip_prefix("inline "))
                .or_else(|| trimmed.strip_prefix("virtual "))
                .or_else(|| trimmed.strip_prefix("const "))
                .unwrap_or(trimmed);
            if let Some(name) = after_keyword(rest, "struct") {
                out.push((name, "Struct"));
            } else if let Some(name) = after_keyword(rest, "class") {
                out.push((name, "Class"));
            } else if let Some(name) = after_keyword(rest, "enum") {
                out.push((name, "Enum"));
            } else if let Some(name) = after_keyword(rest, "using") {
                out.push((name, "Type"));
            } else if !rest.is_empty() && !control_word(rest) {
                let words: Vec<&str> = rest.split_whitespace().collect();
                if words.len() >= 2
                    && let Some(last) = words.last()
                    && !last.starts_with('(')
                {
                    let name = last.trim_matches('*').trim_matches('&');
                    if is_ident(name) {
                        out.push((name.to_string(), "Function"));
                    }
                }
            }
        }
        "java" => {
            if let Some(name) = after_keyword(before, "class") {
                out.push((name, "Class"));
            } else if let Some(name) = after_keyword(before, "interface") {
                out.push((name, "Interface"));
            } else if let Some(name) = after_keyword(before, "enum") {
                out.push((name, "Enum"));
            } else if !before.is_empty() && !control_word(before) {
                let words: Vec<&str> = before.split_whitespace().collect();
                if words.len() >= 2
                    && let Some(last) = words.last()
                    && let Some(first) = words.first()
                    && is_type_like(first)
                {
                    let name = last;
                    if is_ident(name) {
                        out.push((name.to_string(), "Method"));
                    }
                }
            }
        }
        "ruby" => {
            if let Some(name) = after_keyword(before, "def") {
                out.push((name, "Method"));
            } else if let Some(name) = after_keyword(before, "class") {
                out.push((name, "Class"));
            } else if let Some(name) = after_keyword(before, "module") {
                out.push((name, "Module"));
            }
        }
        "php" => {
            if let Some(name) = after_keyword(before, "function") {
                out.push((name, "Function"));
            } else if let Some(name) = after_keyword(before, "class") {
                out.push((name, "Class"));
            } else if let Some(name) = after_keyword(before, "interface") {
                out.push((name, "Interface"));
            }
        }
        _ => {}
    }
    out
}

/// Strip Rust visibility/async/unsafe/extern modifiers from the start of a
/// declaration so the keyword match (`fn`, `struct`, ...) works on `pub fn`,
/// `pub(crate) fn`, `async fn`, `unsafe fn`, `extern "C" fn`.
fn strip_rust_modifiers(mut s: &str) -> &str {
    loop {
        let before = s;
        let mut rest = s.trim_start();
        for m in [
            "pub(crate)",
            "pub(super)",
            "pub(in",
            "pub",
            "async",
            "unsafe",
        ] {
            if let Some(r) = rest.strip_prefix(m) {
                rest = r.trim_start();
                break;
            }
        }
        // `const` only when it qualifies a function (`const fn`), not a const item.
        if let Some(r) = rest.strip_prefix("const") {
            let after = r.trim_start();
            if after.starts_with("fn ") || after.starts_with("async ") {
                rest = after;
            }
        }
        s = rest;
        if s == before {
            break;
        }
    }
    s
}

fn control_word(s: &str) -> bool {
    let first = s.split_whitespace().next().unwrap_or("");
    matches!(
        first,
        "if" | "for" | "while" | "switch" | "return" | "catch" | "else" | "do" | "case"
    )
}

fn is_type_like(word: &str) -> bool {
    word.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
}

fn first_word(s: &str) -> Option<&str> {
    s.split_whitespace()
        .next()
        .map(|w| w.trim_matches(|c| c == '<' || c == '>' || c == '{'))
}

/// Extract the identifier after a keyword like `fn` / `def` / `class`,
/// handling generics (`struct Foo<T>`), traits (`trait X: Y`), and
/// async (`async def f`).
fn after_keyword(s: &str, kw: &str) -> Option<String> {
    let trimmed = s.trim();
    let rest = trimmed.strip_prefix(kw)?;
    let rest = rest.trim_start();
    if rest.is_empty() || rest.starts_with('(') || rest.starts_with('=') {
        return None;
    }
    let name = rest
        .split(|c: char| {
            c.is_whitespace() || c == '<' || c == '(' || c == ':' || c == '=' || c == '['
        })
        .next()?
        .trim_matches(|c: char| c == '"' || c == '\'' || c == ',' || c == ';');
    if name.is_empty() || !is_ident(name) {
        return None;
    }
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_symbols() {
        let content = r#"
pub fn add(a: i32, b: i32) -> i32 { a + b }
struct Point { x: f64 }
enum Color { Red }
trait Draw { fn draw(&self); }
const MAX: usize = 10;
"#;
        let syms = extract_symbols(content, "rust");
        let names: Vec<(&str, &str)> = syms
            .iter()
            .map(|s| (s.name.as_str(), s.kind.as_str()))
            .collect();
        assert!(names.contains(&("add", "Function")));
        assert!(names.contains(&("Point", "Struct")));
        assert!(names.contains(&("Color", "Enum")));
        assert!(names.contains(&("Draw", "Trait")));
        assert!(names.contains(&("MAX", "Const")));
        assert!(syms.iter().all(|s| s.line >= 1));
    }

    #[test]
    fn python_and_go() {
        let py = "def hello(name):\n    return name\nclass Greeter:\n    pass\nasync def fetch():\n    pass";
        let syms = extract_symbols(py, "python");
        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["hello", "Greeter", "fetch"]);

        let go = "func main() {}\nfunc (r *Repo) Find() {}\ntype Config struct {}\ntype Reader interface { Read() }";
        let syms = extract_symbols(go, "go");
        let names: Vec<(&str, &str)> = syms
            .iter()
            .map(|s| (s.name.as_str(), s.kind.as_str()))
            .collect();
        assert!(names.contains(&("main", "Function")));
        assert!(names.contains(&("Find", "Method")));
        assert!(names.contains(&("Config", "Struct")));
        assert!(names.contains(&("Reader", "Interface")));
    }

    #[test]
    fn skips_comments_and_control_keywords() {
        let cpp = "// int fake();\nif (x) {}\nvoid real() {}\nstruct Vec2 {}";
        let syms = extract_symbols(cpp, "cpp");
        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"real"));
        assert!(names.contains(&"Vec2"));
        assert!(!names.contains(&"fake"));
        assert!(!names.contains(&"x"));
    }

    #[test]
    fn function_count_counts_only_functions_and_methods() {
        let src = "pub struct S;\nimpl S {\n    pub fn method(&self) {}\n}\nfn helper() {}\nconst C: u32 = 1;\n";
        assert_eq!(function_count(src, "rust"), 2);
        assert_eq!(function_count(src, "python"), 0); // wrong lang -> nothing
    }

    #[test]
    fn function_count_zero_for_unsupported_language() {
        assert_eq!(function_count("def f(): pass", "plaintext"), 0);
    }
}
