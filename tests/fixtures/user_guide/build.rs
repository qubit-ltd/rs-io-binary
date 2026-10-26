// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn normalize(source: &str) -> String {
    source
        .replace("\r\n", "\n")
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

/// Extracts the explicitly marked README async example.
fn extract_readme_async(markdown: &str) -> Result<String, String> {
    const START: &str = "<!-- readme-async:start -->";
    const END: &str = "<!-- readme-async:end -->";
    let markdown = markdown.replace("\r\n", "\n");
    if markdown.matches(START).count() != 1 {
        return Err("expected exactly one async start marker".to_owned());
    }
    if markdown.matches(END).count() != 1 {
        return Err("expected exactly one async end marker".to_owned());
    }
    let start = markdown.find(START).expect("one start marker");
    let end = markdown.find(END).expect("one end marker");
    if start >= end {
        return Err("async start marker must precede end marker".to_owned());
    }
    let contents = markdown[start + START.len()..end].trim();
    let contents = contents
        .strip_prefix("```rust\n")
        .ok_or_else(|| "missing async rust fence".to_owned())?;
    let contents = contents
        .strip_suffix("\n```")
        .ok_or_else(|| "missing async closing fence".to_owned())?;
    if contents.contains("```") {
        return Err("unexpected extra async fence".to_owned());
    }
    Ok(normalize(contents))
}

/// Requires both README languages to compile the same async source.
fn readme_async_pair(english: &str, chinese: &str) -> Result<String, String> {
    let english = extract_readme_async(english)
        .map_err(|error| format!("English README: {error}"))?;
    let chinese = extract_readme_async(chinese)
        .map_err(|error| format!("Chinese README: {error}"))?;
    if english != chinese {
        return Err("README async code differs between languages".to_owned());
    }
    Ok(english)
}

fn rust_blocks(markdown: &str) -> Result<Vec<String>, String> {
    let markdown = markdown.replace("\r\n", "\n");
    let mut blocks = Vec::new();
    let mut current: Option<Vec<&str>> = None;
    let mut in_other_block = false;

    for line in markdown.lines() {
        if let Some(lines) = current.as_mut() {
            if line == "```" {
                blocks.push(lines.join("\n"));
                current = None;
            } else {
                lines.push(line);
            }
        } else if in_other_block {
            if line == "```" {
                in_other_block = false;
            }
        } else if line == "```rust" {
            current = Some(Vec::new());
        } else if line.starts_with("```") {
            in_other_block = true;
        }
    }

    if current.is_some() || in_other_block {
        return Err("unclosed fenced code block".to_owned());
    }
    if blocks.len() != 6 {
        return Err(format!("expected exactly 6 Rust blocks, found {}", blocks.len()));
    }
    Ok(blocks.into_iter().map(|block| normalize(&block)).collect())
}

fn read_guide(path: &Path) -> Vec<String> {
    rust_blocks(&fs::read_to_string(path).expect("read user guide"))
        .expect("extract six Rust examples from user guide")
}

fn main() {
    let fixture = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("fixture directory"));
    let root = fixture.join("../../..");
    let english_path = root.join("doc/user_guide.md");
    let chinese_path = root.join("doc/user_guide.zh_CN.md");
    println!("cargo:rerun-if-changed={}", english_path.display());
    println!("cargo:rerun-if-changed={}", chinese_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    let english_readme_path = root.join("README.md");
    let chinese_readme_path = root.join("README.zh_CN.md");
    println!("cargo:rerun-if-changed={}", english_readme_path.display());
    println!("cargo:rerun-if-changed={}", chinese_readme_path.display());
    let english_readme = fs::read_to_string(&english_readme_path)
        .expect("read English README for async example");
    let chinese_readme = fs::read_to_string(&chinese_readme_path)
        .expect("read Chinese README for async example");
    let readme_async = readme_async_pair(&english_readme, &chinese_readme)
        .expect("extract matching README async examples");

    let english_blocks = read_guide(&english_path);
    let chinese_blocks = read_guide(&chinese_path);
    assert_eq!(english_blocks, chinese_blocks, "user guide Rust examples differ between languages");

    let mut generated = String::new();
    for (index, source) in english_blocks.iter().enumerate() {
        generated.push_str(&format!("mod example_{index} {{\n{source}\n"));
        if index != 4 {
            generated.push_str("pub fn run() -> std::io::Result<()> { main() }\n");
        }
        generated.push_str("}\n");
    }
    generated.push_str(&format!("mod readme_async_example {{\n{readme_async}\n}}\n"));
    generated.push_str("fn main() -> std::io::Result<()> {\n");
    for index in [0, 1, 2, 3, 5] {
        generated.push_str(&format!("example_{index}::run()?;\n"));
    }
    generated.push_str("Ok(())\n}\n");

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    fs::write(output.join("user_guide_examples.rs"), generated)
        .expect("write generated user guide examples");
}

#[cfg(test)]
mod tests {
    use super::extract_readme_async;
    use super::readme_async_pair;

    const START: &str = "<!-- readme-async:start -->";
    const END: &str = "<!-- readme-async:end -->";

    fn markdown(source: &str) -> String {
        format!("{START}\n```rust\n{source}\n```\n{END}")
    }

    #[test]
    fn test_extracts_and_normalizes_async_source() {
        let expected = "async fn example() {}";
        assert_eq!(extract_readme_async(&markdown(expected)).unwrap(), expected);
        let crlf = markdown("async fn example() {}  ").replace('\n', "\r\n");
        assert_eq!(extract_readme_async(&crlf).unwrap(), expected);
    }

    #[test]
    fn test_rejects_invalid_marker_and_fence_layouts() {
        let valid = markdown("async fn example() {}");
        let cases = [
            (valid.replace(START, ""), "expected exactly one async start marker"),
            (valid.replace(END, ""), "expected exactly one async end marker"),
            (format!("{START}\n{valid}"), "expected exactly one async start marker"),
            (format!("{valid}\n{END}"), "expected exactly one async end marker"),
            (
                format!("{END}\n```rust\nx\n```\n{START}"),
                "async start marker must precede end marker",
            ),
            (valid.replace("```rust", "```text"), "missing async rust fence"),
            (
                format!("{START}\n```rust\nx\n{END}"),
                "missing async closing fence",
            ),
            (markdown("x\n```\n```rust\ny"), "unexpected extra async fence"),
        ];
        for (source, expected) in cases {
            assert_eq!(extract_readme_async(&source).unwrap_err(), expected);
        }
    }

    #[test]
    fn test_checks_bilingual_async_source() {
        let source = markdown("async fn example() {}");
        assert_eq!(
            readme_async_pair(&source, &source).unwrap(),
            "async fn example() {}"
        );
        let different = markdown("async fn another_example() {}");
        assert_eq!(
            readme_async_pair(&source, &different).unwrap_err(),
            "README async code differs between languages"
        );
        assert_eq!(
            readme_async_pair("", &source).unwrap_err(),
            "English README: expected exactly one async start marker"
        );
        assert_eq!(
            readme_async_pair(&source, "").unwrap_err(),
            "Chinese README: expected exactly one async start marker"
        );
    }
}
