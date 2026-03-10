use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(err) = generate_levels_manifest() {
        panic!("failed to generate assets/levels.json: {err}");
    }
}

fn generate_levels_manifest() -> io::Result<()> {
    let levels_dir = Path::new("assets/levels");
    let output_file = Path::new("assets/levels.json");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", levels_dir.display());

    if !levels_dir.exists() {
        // Keep builds working even if no levels are present yet.
        fs::write(output_file, "[]\n")?;
        return Ok(());
    }

    let level_files: Vec<PathBuf> = fs::read_dir(levels_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();

    let mut ordered_files = Vec::with_capacity(level_files.len());
    for file in level_files {
        let order = parse_order_prefix(&file).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Level file '{}' must be prefixed like '001_name.json'",
                    file.display()
                ),
            )
        })?;
        ordered_files.push((order, file));
    }

    ordered_files.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut levels = Vec::with_capacity(ordered_files.len());
    let mut level_names = HashSet::new();

    for (_, file) in &ordered_files {
        println!("cargo:rerun-if-changed={}", file.display());
        let contents = fs::read_to_string(file)?;
        let value: serde_json::Value = serde_json::from_str(&contents).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {}", file.display(), e),
            )
        })?;
        let level_name = value
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{}: missing string field 'name'", file.display()),
                )
            })?
            .to_string();

        if !level_names.insert(level_name.clone()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "duplicate level name '{}'; level names must be unique",
                    level_name
                ),
            ));
        }

        levels.push(value);
    }

    let output =
        serde_json::to_string_pretty(&levels).map_err(|e| io::Error::other(e.to_string()))?;

    fs::write(output_file, format!("{output}\n"))?;
    Ok(())
}

fn parse_order_prefix(path: &Path) -> Option<u32> {
    let stem = path.file_stem()?.to_str()?;
    let (prefix, rest) = stem.split_once('_')?;
    if prefix.len() != 3 || !prefix.chars().all(|c| c.is_ascii_digit()) || rest.is_empty() {
        return None;
    }
    prefix.parse().ok()
}
