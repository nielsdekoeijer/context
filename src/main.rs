use clap::{Parser, ValueEnum};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(ValueEnum, Debug, Clone)]
enum Filter {
    All,
    Rust,
    Cpp,
    Nix,
    C,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    filters: Option<Vec<Filter>>,

    /// Wildcard patterns to exclude (e.g., "test*", "target", "*.txt")
    #[arg(short, long)]
    exclude: Option<Vec<String>>,
}

const EXTENSIONS_BASE: &[&str] = &["txt", "md"];
const EXTENSIONS_RUST: &[&str] = &["rs", "toml"];
const EXTENSIONS_CPP: &[&str] = &["cpp", "hpp", "cxx"];
const EXTENSIONS_NIX: &[&str] = &["nix"];
const EXTENSIONS_C: &[&str] = &["c", "h"];

fn read_files_recursive(
    path: &Path,
    extensions: &[&str],
    excludes: &GlobSet,
    filtered_files: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    if path.is_dir() {
        for entry_result in std::fs::read_dir(path)? {
            let entry = entry_result?;
            let path = entry.path();

            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            if excludes.is_match(file_name.as_ref()) {
                continue;
            }

            if path.is_dir() {
                read_files_recursive(&path, &extensions, &excludes, &mut *filtered_files)?;
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext) {
                        filtered_files.push(path);
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let filters = args.filters.unwrap_or(vec![Filter::All]);
    let mut extensions = Vec::<&str>::new();
    let exclude_patterns = args.exclude.unwrap_or_default();

    for filter in filters {
        match filter {
            Filter::All => {
                extensions.extend(EXTENSIONS_BASE);
                extensions.extend(EXTENSIONS_RUST);
                extensions.extend(EXTENSIONS_CPP);
                extensions.extend(EXTENSIONS_NIX);
                extensions.extend(EXTENSIONS_C);
            }
            Filter::Rust => {
                extensions.extend(EXTENSIONS_BASE);
                extensions.extend(EXTENSIONS_RUST);
            }
            Filter::Nix => {
                extensions.extend(EXTENSIONS_BASE);
                extensions.extend(EXTENSIONS_NIX);
            }
            Filter::Cpp => {
                extensions.extend(EXTENSIONS_BASE);
                extensions.extend(EXTENSIONS_CPP);
                extensions.extend(EXTENSIONS_C);
            }
            Filter::C => {
                extensions.extend(EXTENSIONS_BASE);
                extensions.extend(EXTENSIONS_C);
            }
        }
    }

    let pwd = Path::new(".");
    let mut filtered_files = Vec::<PathBuf>::new();

    let mut builder = GlobSetBuilder::new();
    for pattern in exclude_patterns {
        builder.add(Glob::new(&pattern)?);
    }
    let excludes = builder.build()?;

    read_files_recursive(&pwd, &extensions, &excludes, &mut filtered_files)?;

    let editor = std::env::var("EDITOR")?;

    let mut child = std::process::Command::new(&editor)
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            eprintln!(
                "Failure to start piping into editor `{}`",
                editor.to_string()
            );
            e
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        for file in &filtered_files {
            writeln!(stdin, "\n// ==== {} ====", file.display())?;

            match File::open(&file) {
                Ok(mut f) => {
                    std::io::copy(&mut f, &mut stdin)?;
                }
                Err(e) => {
                    eprintln!("Failed to read {}: {}", file.display(), e);
                }
            }
        }
    }

    child.wait()?;

    Ok(())
}
