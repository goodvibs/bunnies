use std::fs;
use std::path::Path;

fn to_snake_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn exported_primary_type_names(contents: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut depth = 0usize;
    for line in contents.lines() {
        let trimmed = line.trim_start();

        if depth == 0 && !trimmed.starts_with("//") && !trimmed.starts_with("pub(crate)") {
            let marker = if trimmed.starts_with("pub struct ") {
                Some("pub struct ")
            } else if trimmed.starts_with("pub enum ") {
                Some("pub enum ")
            } else if trimmed.starts_with("pub union ") {
                Some("pub union ")
            } else if trimmed.starts_with("pub type ") {
                Some("pub type ")
            } else {
                None
            };

            if let Some(marker) = marker {
                let rest = &trimmed[marker.len()..];
                let name = rest
                    .split(|c: char| {
                        c == '<' || c == ':' || c == '=' || c == ';' || c.is_whitespace()
                    })
                    .next()
                    .unwrap_or_default();
                if !name.is_empty() {
                    names.push(name.to_string());
                }
            }
        }

        depth += line.chars().filter(|&c| c == '{').count();
        depth = depth.saturating_sub(line.chars().filter(|&c| c == '}').count());
    }
    names
}

fn exported_trait_names(contents: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut depth = 0usize;
    for line in contents.lines() {
        let trimmed = line.trim_start();

        if depth == 0 && !trimmed.starts_with("//") && !trimmed.starts_with("pub(crate)") {
            let marker = if trimmed.starts_with("pub trait ") {
                Some("pub trait ")
            } else if trimmed.starts_with("pub const trait ") {
                Some("pub const trait ")
            } else {
                None
            };

            if let Some(marker) = marker {
                let rest = &trimmed[marker.len()..];
                let name = rest
                    .split(|c: char| {
                        c == '<' || c == ':' || c == '=' || c == ';' || c.is_whitespace()
                    })
                    .next()
                    .unwrap_or_default();
                if !name.is_empty() {
                    names.push(name.to_string());
                }
            }
        }

        depth += line.chars().filter(|&c| c == '{').count();
        depth = depth.saturating_sub(line.chars().filter(|&c| c == '}').count());
    }
    names
}

#[test]
fn types_module_uses_one_exported_type_per_file_with_matching_name() {
    let types_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/types");
    let mut violations: Vec<String> = Vec::new();

    for entry in fs::read_dir(&types_dir).expect("failed to read types dir") {
        let entry = entry.expect("failed to read dir entry");
        let path = entry.path();

        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if filename == "mod.rs" {
            continue;
        }

        let contents = fs::read_to_string(&path).expect("failed to read source file");
        let exported_types = exported_primary_type_names(&contents);
        let declared = if exported_types.len() == 1 {
            exported_types[0].clone()
        } else if exported_types.is_empty() {
            let exported_traits = exported_trait_names(&contents);
            if exported_traits.len() == 1 {
                exported_traits[0].clone()
            } else {
                violations.push(format!(
                    "{filename}: expected exactly 1 exported primary type or 1 exported trait, found primary={:?}, trait={:?}",
                    exported_types, exported_traits
                ));
                continue;
            }
        } else {
            violations.push(format!(
                "{filename}: expected exactly 1 exported primary type, found {} ({exported_types:?})",
                exported_types.len()
            ));
            continue;
        };

        let expected_filename = format!("{}.rs", to_snake_case(&declared));
        if filename != expected_filename {
            violations.push(format!(
                "{filename}: exported type `{declared}` should live in `{expected_filename}`"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "types module export/file violations:\n{}",
        violations.join("\n")
    );
}
