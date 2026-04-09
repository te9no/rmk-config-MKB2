use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

fn expand_config_includes(path: &Path) -> std::io::Result<String> {
    let mut visited = Vec::new();
    expand_config_includes_inner(path, &mut visited)
}

fn collect_config_dependencies(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut visited = Vec::new();
    collect_config_dependencies_inner(path, &mut visited)?;
    Ok(visited)
}

fn expand_config_includes_inner(path: &Path, visited: &mut Vec<PathBuf>) -> std::io::Result<String> {
    let canonical = fs::canonicalize(path)?;
    if visited.iter().any(|p| p == &canonical) {
        return Err(std::io::Error::other(format!(
            "circular config include detected: {}",
            canonical.display()
        )));
    }

    visited.push(canonical.clone());

    let content = fs::read_to_string(&canonical)?;
    let mut expanded = String::new();
    let parent = canonical.parent().unwrap_or_else(|| Path::new("."));

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(include_path) = trimmed.strip_prefix("# include:") {
            let include_path = parent.join(include_path.trim());
            expanded.push_str(&expand_config_includes_inner(&include_path, visited)?);
            if !expanded.ends_with('\n') {
                expanded.push('\n');
            }
        } else {
            expanded.push_str(line);
            expanded.push('\n');
        }
    }

    visited.pop();
    Ok(expanded)
}

fn collect_config_dependencies_inner(path: &Path, visited: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let canonical = fs::canonicalize(path)?;
    if visited.iter().any(|p| p == &canonical) {
        return Ok(());
    }

    visited.push(canonical.clone());
    let content = fs::read_to_string(&canonical)?;
    let parent = canonical.parent().unwrap_or_else(|| Path::new("."));

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(include_path) = trimmed.strip_prefix("# include:") {
            let include_path = parent.join(include_path.trim());
            collect_config_dependencies_inner(&include_path, visited)?;
        }
    }

    Ok(())
}

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    println!("cargo:warning=build.rs: OUT_DIR={}", out.display());
    println!("cargo:warning=build.rs: MANIFEST_DIR={}", manifest_dir.display());
    println!("cargo:rerun-if-env-changed=MODULE");

    // memory.xをコピー
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tlink.x");
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=build.rs");

    // モジュールに基づいてkeyboard.tomlをコピー
    let module = env::var("MODULE").unwrap_or_else(|_| "default".to_string());
    let keyboard_toml_path = manifest_dir.join(format!("config/{}.toml", module));
    let keyboard_toml_out = manifest_dir.join("keyboard.toml");
    
    println!("cargo:warning=build.rs: MODULE={}, source={}", module, keyboard_toml_path.display());
    
    if keyboard_toml_path.exists() {
        for dep in collect_config_dependencies(&keyboard_toml_path).unwrap() {
            println!("cargo:rerun-if-changed={}", dep.display());
        }
        let expanded = expand_config_includes(&keyboard_toml_path).unwrap();
        fs::write(&keyboard_toml_out, expanded).unwrap();
        println!("cargo:warning=build.rs: Copied {} to keyboard.toml", module);
    } else {
        println!("cargo:warning=build.rs: {} not found, using existing keyboard.toml", keyboard_toml_path.display());
        println!("cargo:rerun-if-changed={}", keyboard_toml_out.display());
    }
}
