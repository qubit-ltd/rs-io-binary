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
    generated.push_str("fn main() -> std::io::Result<()> {\n");
    for index in [0, 1, 2, 3, 5] {
        generated.push_str(&format!("example_{index}::run()?;\n"));
    }
    generated.push_str("Ok(())\n}\n");

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    fs::write(output.join("user_guide_examples.rs"), generated)
        .expect("write generated user guide examples");
}
