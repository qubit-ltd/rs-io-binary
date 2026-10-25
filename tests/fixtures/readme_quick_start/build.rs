// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generates the executable quick-start example from both language READMEs.

use std::env;
use std::fs;
use std::path::PathBuf;

/// Extracts the marked Rust quick-start block and normalizes its line endings.
fn extract_quick_start(markdown: &str) -> Result<String, String> {
    const START: &str = "<!-- readme-quick-start:start -->";
    const END: &str = "<!-- readme-quick-start:end -->";

    let markdown = markdown.replace("\r\n", "\n");
    if markdown.matches(START).count() != 1 {
        return Err("expected exactly one start marker".to_owned());
    }
    if markdown.matches(END).count() != 1 {
        return Err("expected exactly one end marker".to_owned());
    }

    let start = markdown.find(START).expect("one start marker");
    let end = markdown.find(END).expect("one end marker");
    if start >= end {
        return Err("start marker must precede end marker".to_owned());
    }
    let contents = markdown[start + START.len()..end].trim();
    let Some(contents) = contents.strip_prefix("```rust\n") else {
        return Err("missing rust fence".to_owned());
    };
    let Some(contents) = contents.strip_suffix("\n```") else {
        return Err("missing closing fence".to_owned());
    };
    if contents.contains("```") {
        return Err("unexpected extra fence".to_owned());
    }

    Ok(contents
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned())
}

fn main() {
    let fixture = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("fixture directory"));
    let root = fixture.join("../../..");
    let english_path = root.join("README.md");
    let chinese_path = root.join("README.zh_CN.md");
    println!("cargo:rerun-if-changed={}", english_path.display());
    println!("cargo:rerun-if-changed={}", chinese_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    let english = extract_quick_start(&fs::read_to_string(&english_path).expect("read English README"))
        .expect("English README quick start");
    let chinese = extract_quick_start(&fs::read_to_string(&chinese_path).expect("read Chinese README"))
        .expect("Chinese README quick start");
    assert_eq!(english, chinese, "README quick-start code differs between languages");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    fs::write(output.join("readme_quick_start.rs"), english).expect("write generated quick start");
}

#[cfg(test)]
mod tests {
    use super::extract_quick_start;

    const START: &str = "<!-- readme-quick-start:start -->";
    const END: &str = "<!-- readme-quick-start:end -->";

    fn markdown(block: &str) -> String {
        format!("{START}\n{block}\n{END}")
    }

    #[test]
    fn extracts_normal_example_and_preserves_comments() {
        let source = markdown("```rust\n// ordinary comment\nlet value = 1;\n```");
        assert_eq!(
            extract_quick_start(&source).expect("extract block"),
            "// ordinary comment\nlet value = 1;"
        );
    }

    #[test]
    fn normalizes_crlf_and_trims_line_ends() {
        let source = format!("{START}\r\n```rust\r\nlet value = 1;  \r\n```\r\n{END}");
        assert_eq!(extract_quick_start(&source).expect("extract block"), "let value = 1;");
    }

    #[test]
    fn preserves_rustdoc_hidden_prefixes_as_source() {
        let source = markdown("```rust\n# Ok::<(), Error>(())\n```");
        assert_eq!(
            extract_quick_start(&source).expect("extract block"),
            "# Ok::<(), Error>(())"
        );
    }

    #[test]
    fn rejects_missing_or_duplicate_markers() {
        assert_eq!(
            extract_quick_start("no markers").expect_err("missing markers"),
            "expected exactly one start marker"
        );
        assert_eq!(
            extract_quick_start(&format!("{START}\n```rust\nx\n```")).expect_err("missing end marker"),
            "expected exactly one end marker"
        );
        assert_eq!(
            extract_quick_start(&format!("{START}\n{START}\n```rust\nx\n```\n{END}")).expect_err("duplicate start"),
            "expected exactly one start marker"
        );
        assert_eq!(
            extract_quick_start(&format!("{START}\n```rust\nx\n```\n{END}\n{END}")).expect_err("duplicate end"),
            "expected exactly one end marker"
        );
    }

    #[test]
    fn rejects_reversed_markers() {
        let source = format!("{END}\n```rust\nx\n```\n{START}");
        assert_eq!(
            extract_quick_start(&source).expect_err("reversed markers"),
            "start marker must precede end marker"
        );
    }

    #[test]
    fn rejects_wrong_language_fence() {
        assert_eq!(
            extract_quick_start(&markdown("```text\nx\n```")).expect_err("wrong language"),
            "missing rust fence"
        );
    }

    #[test]
    fn rejects_unclosed_fence() {
        assert_eq!(
            extract_quick_start(&markdown("```rust\nx")).expect_err("unclosed fence"),
            "missing closing fence"
        );
    }

    #[test]
    fn rejects_extra_fence() {
        assert_eq!(
            extract_quick_start(&markdown("```rust\nx\n```\n```rust\ny\n```")).expect_err("extra fence"),
            "unexpected extra fence"
        );
    }
}
