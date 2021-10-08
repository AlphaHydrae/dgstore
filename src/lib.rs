use ansi_term::Colour;
use ansi_term::Colour::Cyan;
use ansi_term::Colour::Green;
use ansi_term::Colour::Red;
use ansi_term::Colour::Yellow;
use ansi_term::Style;
use glob::glob;
use glob::GlobError;
use glob::PatternError;
use sha2::{Digest, Sha512};
use std::env;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process;

pub struct Config {
    patterns: Vec<String>,
    write: bool,
}

impl Config {
    pub fn new(patterns: Vec<String>, write: bool) -> Result<Config, DgStoreError> {
        if patterns.is_empty() {
            return Err(DgStoreError::NoPatternsSpecified);
        }

        Ok(Config { patterns, write })
    }
}

pub enum DgStoreError {
    Digest(io::Error),
    Glob(GlobError),
    GlobPattern(String, PatternError),
    NoPatternsSpecified,
}

impl fmt::Display for DgStoreError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DgStoreError::Digest(error) => write!(f, "Digest error: {}", error),
            DgStoreError::Glob(error) => write!(f, "Path glob error: {}", error),
            DgStoreError::GlobPattern(pattern, error) => {
                write!(f, "Pattern {} is invalid: {}", pattern, error)
            }
            DgStoreError::NoPatternsSpecified => {
                write!(f, "At least one file argument is required")
            }
        }
    }
}

pub fn compute_and_store_digests(config: Config) -> Result<(), DgStoreError> {
    for file in glob_files(config.patterns)? {
        if fs::metadata(&file).map_or(false, |meta| meta.is_dir()) {
            continue;
        }

        match hash_file(&file, config.write) {
            Ok(_) => {}
            Err(error) => {
                println!(
                    "{}",
                    paint(
                        Red,
                        format!("Could not hash file {}: {}", file.display(), error)
                    )
                );
                process::exit(1);
            }
        }
    }

    Ok(())
}

fn glob_files(patterns: Vec<String>) -> Result<Vec<PathBuf>, DgStoreError> {
    let valid_patterns = execute_glob_patterns(patterns)?;

    let mut files: Vec<PathBuf> = Vec::new();
    for pattern in valid_patterns {
        for result in pattern {
            match result {
                Ok(file) => {
                    files.push(file.as_path().to_path_buf());
                }
                Err(err) => {
                    return Err(DgStoreError::Glob(err));
                }
            }
        }
    }

    Ok(files)
}

fn execute_glob_patterns(patterns: Vec<String>) -> Result<Vec<glob::Paths>, DgStoreError> {
    let mut results: Vec<glob::Paths> = Vec::new();
    for pattern in patterns {
        match glob(&pattern) {
            Ok(paths) => results.push(paths),
            Err(err) => return Err(DgStoreError::GlobPattern(pattern.to_string(), err)),
        }
    }

    Ok(results)
}

fn hash_file(path: &Path, write: bool) -> Result<(), DgStoreError> {
    let mut file = File::open(path).map_err(DgStoreError::Digest)?;

    let digest_file_path_buf = add_extension(path, ".sha512");
    let digest_file_path = digest_file_path_buf.as_path();
    let digest_file_contents = read_digest_file(digest_file_path)?;

    let mut hasher = Sha512::new();

    io::copy(&mut file, &mut hasher).map_err(DgStoreError::Digest)?;

    let actual_digest = format!("{:x}", hasher.finalize());

    match digest_file_contents.map(|digest| {
        let same = digest == actual_digest;
        (digest, same)
    }) {
        Some((expected_digest, true)) => show_file_unchanged(path, &expected_digest),
        Some((expected_digest, false)) => show_file_changed(path, &actual_digest, &expected_digest),
        None => {
            if write {
                save_digest(path, digest_file_path, &actual_digest)
            } else {
                show_digest(path, &actual_digest)
            }
        }
    }
}

fn save_digest(path: &Path, digest_path: &Path, digest: &str) -> Result<(), DgStoreError> {
    fs::write(&digest_path, &digest).map_err(DgStoreError::Digest)?;

    println!(
        "{} {} {} {}",
        paint(Green, "✓"),
        paint(Cyan, format!("{:.7}", digest)),
        path.display(),
        paint(
            Yellow,
            format!("(stored digest to {})", digest_path.display())
        )
    );

    Ok(())
}

fn show_digest(path: &Path, digest: &str) -> Result<(), DgStoreError> {
    println!(
        "{} {} {} (not saving digest)",
        paint(Green, "✓"),
        paint(Cyan, format!("{:.7}", digest)),
        path.display()
    );

    Ok(())
}

fn show_file_unchanged(path: &Path, digest: &str) -> Result<(), DgStoreError> {
    println!(
        "{} {} {}",
        paint(Green, "✓"),
        paint(Green, format!("{:.7}", digest)),
        path.display()
    );

    Ok(())
}

fn show_file_changed(path: &Path, digest: &str, original_digest: &str) -> Result<(), DgStoreError> {
    println!(
        "{} {} {} {}",
        paint(Red, "✗"),
        paint(Red, format!("{:.7}", digest)),
        path.display(),
        paint(
            Yellow,
            format!(
                "(previous digest was {})",
                format!("{:.7}", original_digest)
            )
            .as_str()
        )
    );

    Ok(())
}

fn read_digest_file(path: &Path) -> Result<Option<String>, DgStoreError> {
    fs::read_to_string(path)
        .map(Some)
        .or_else(|err| match err.kind() {
            io::ErrorKind::NotFound => Ok(None),
            _ => Err(DgStoreError::Digest(err)),
        })
}

fn paint<T>(color: Colour, contents: T) -> String
where
    T: Into<String>,
{
    match env::var("NO_COLOR") {
        Ok(_) => contents.into(),
        Err(_) => Style::from(color).paint(contents.into()).to_string(),
    }
}

fn add_extension(path: &Path, extension: &str) -> PathBuf {
    let mut path_string = path.as_os_str().to_os_string();
    path_string.push(extension);
    Path::new(&path_string).to_path_buf()
}
