//! Enforces the hexagonal layer rules described in AGENTS.md
//!
//! Each layer may only reference certain modules and crates. A reference is a
//! path segment such as `axum::` or `infra::`, so `use crate::infra::X`,
//! `super::super::infra::X` and `use crate::{infra::X}` are all caught.
//! `#[cfg(test)]` modules at the end of a file are exempt.

use std::fs;
use std::path::{Path, PathBuf};

struct Layer {
    dir: &'static str,
    forbidden: &'static [&'static str],
    why: &'static str,
}

const HTTP: [&str; 4] = ["axum", "axum_core", "axum_extra", "tower_http"];

const LAYERS: &[Layer] = &[
    Layer {
        dir: "src/domain",
        forbidden: &[
            "app",
            "infra",
            "presentation",
            "bootstrap",
            HTTP[0],
            HTTP[1],
            HTTP[2],
            HTTP[3],
            "sea_orm",
            "jsonwebtoken",
            "utoipa",
            "tokio",
        ],
        why: "the domain is pure business logic; depend on nothing outside domain/",
    },
    Layer {
        dir: "src/app",
        forbidden: &[
            "infra",
            "presentation",
            "bootstrap",
            HTTP[0],
            HTTP[1],
            HTTP[2],
            HTTP[3],
            "sea_orm",
            "jsonwebtoken",
        ],
        why: "use cases are transport- and storage-agnostic; define a port in app/ports and implement it in infra/",
    },
    Layer {
        dir: "src/infra",
        forbidden: &[
            "presentation",
            "bootstrap",
            HTTP[0],
            HTTP[1],
            HTTP[2],
            HTTP[3],
            "utoipa",
        ],
        why: "adapters implement domain/app ports and know nothing about HTTP",
    },
    Layer {
        dir: "src/presentation",
        forbidden: &["persistence", "bootstrap", "sea_orm", "jsonwebtoken"],
        why: "handlers reach storage and tokens only through use cases in AppState",
    },
];

#[test]
fn layers_only_depend_inward() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut violations = Vec::new();

    for layer in LAYERS {
        let files = rust_files(&root.join(layer.dir));
        assert!(!files.is_empty(), "no Rust files found in {}", layer.dir);

        for file in files {
            let source = fs::read_to_string(&file).unwrap();
            for (line_no, segment) in references(&source, layer.forbidden) {
                violations.push(format!(
                    "{}:{line_no}: `{segment}::` is not allowed in {}: {}",
                    file.strip_prefix(&root).unwrap().display(),
                    layer.dir,
                    layer.why,
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "layer rule violations (see AGENTS.md > Architecture):\n{}",
        violations.join("\n")
    );
}

#[test]
fn reference_scanner_catches_every_import_style() {
    let source = "\
use crate::infra::Config;
use crate::{domain::User, infra::persistence::Repo};
let x = super::super::infra::thing();
// infra::mentioned_in_a_comment
let infrastructure = 1;
#[cfg(test)]
mod tests { use crate::infra::Test; }
";
    let lines: Vec<usize> = references(source, &["infra"])
        .into_iter()
        .map(|(line, _)| line)
        .collect();
    assert_eq!(lines, vec![1, 2, 3]);
}

/// Find `(line number, segment)` for each forbidden path segment in the source
fn references(source: &str, forbidden: &[&str]) -> Vec<(usize, String)> {
    let mut found = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let code = line.trim_start();
        if code.starts_with("#[cfg(test)]") {
            break;
        }
        if code.starts_with("//") {
            continue;
        }

        for segment in forbidden {
            if contains_segment(code, segment) {
                found.push((index + 1, segment.to_string()));
            }
        }
    }

    found
}

/// Whether `segment::` appears as a whole path segment in `code`
fn contains_segment(code: &str, segment: &str) -> bool {
    let pattern = format!("{segment}::");
    code.match_indices(&pattern).any(|(start, _)| {
        let before = code[..start].chars().next_back();
        !before.is_some_and(|c| c.is_alphanumeric() || c == '_')
    })
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(rust_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files
}
