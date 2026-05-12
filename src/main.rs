use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};
use std::fs::{File};
use std::io::Write;

#[derive(ValueEnum, Debug, Clone)]
enum Filter {
    Smart,
    Rust,
    Cpp,
    C,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    filters: Option<Vec<Filter>>,
}

fn read_files_recursive(
    path: &Path,
    extensions: &[&str],
    filtered_files: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    if path.is_dir() {
        for entry_result in std::fs::read_dir(path)? {
            let entry = entry_result?;
            let path = entry.path();

            if path.is_dir() {
                read_files_recursive(&path, &extensions, &mut *filtered_files)?;
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
    let filters = args.filters.unwrap_or(vec![Filter::Smart]);
    let mut extensions = vec!["md"];

    for filter in filters {
        match filter {
            Filter::Smart => {
                unimplemented!("Smart filtering to be implemented at a later date");
            }
            Filter::Rust => extensions.extend(vec!["rs", "toml"]),
            Filter::Cpp => extensions.extend(vec!["cpp", "hpp", "cxx", "c", "h"]),
            Filter::C => extensions.extend(vec!["c", "h"]),
        }
    }

    let pwd = Path::new(".");
    let mut filtered_files = Vec::<PathBuf>::new();

    read_files_recursive(&pwd, &extensions, &mut filtered_files)?;

    let editor = std::env::var("EDITOR")?;

    let mut child = std::process::Command::new(&editor)
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn().map_err(|e| {
            eprintln!("Failure to start piping into editor `{}`", editor.to_string());
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
