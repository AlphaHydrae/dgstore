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
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::process;

enum DgStoreError {
    Digest(io::Error),
    Glob(GlobError),
    GlobPattern(String, PatternError),
    InvalidFilePath(OsString),
}

impl fmt::Display for DgStoreError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DgStoreError::Digest(error) => write!(f, "Digest error: {}", error),
            DgStoreError::InvalidFilePath(path) => {
                write!(f, "Invalid file path matched: {}", path.to_string_lossy())
            }
            DgStoreError::Glob(error) => write!(f, "Path glob error: {}", error),
            DgStoreError::GlobPattern(pattern, error) => {
                write!(f, "Pattern {} is invalid: {}", pattern, error)
            }
        }
    }
}

fn main() {
    let patterns: Vec<String> = env::args().skip(1).collect();
    if patterns.is_empty() {
        println!("{}", paint(Red, "At least one file argument is required"));
        process::exit(1);
    }

    match compute_and_store_digests(&patterns) {
        Ok(_) => {
            process::exit(0);
        }
        Err(error) => {
            println!("{}", paint(Red, format!("{}", error)));
            process::exit(2);
        }
    }
}

fn compute_and_store_digests(patterns: &[String]) -> Result<(), DgStoreError> {
    for file in glob_files(patterns)? {
        if fs::metadata(&file).map_or(false, |meta| meta.is_dir()) {
            continue;
        }

        match hash_file(&file) {
            Ok(_) => {}
            Err(error) => {
                println!(
                    "{}",
                    paint(Red, format!("Could not hash file {}: {}", file, error))
                );
                process::exit(1);
            }
        }
    }

    Ok(())
}

fn glob_files(patterns: &[String]) -> Result<Vec<String>, DgStoreError> {
    let mut files: Vec<String> = Vec::new();
    for pattern in patterns {
        let results =
            glob(pattern).map_err(|err| DgStoreError::GlobPattern(pattern.clone(), err))?;
        for result in results {
            match result {
                Ok(file) => match file.to_str() {
                    Some(path) => {
                        files.push(path.to_string());
                    }
                    None => {
                        let os_str = file.as_path().as_os_str().to_os_string();
                        return Err(DgStoreError::InvalidFilePath(os_str));
                    }
                },
                Err(err) => {
                    return Err(DgStoreError::Glob(err));
                }
            }
        }
    }

    Ok(files)
}

fn hash_file(path: &str) -> Result<(), DgStoreError> {
    let mut file = File::open(path).map_err(DgStoreError::Digest)?;

    let digest_file_path = format!("{}.sha512", path);
    let digest_file_contents = read_digest_file(&digest_file_path)?;

    let mut hasher = Sha512::new();

    io::copy(&mut file, &mut hasher).map_err(DgStoreError::Digest)?;

    let actual_digest = format!("{:x}", hasher.finalize());

    match digest_file_contents.map(|digest| {
        let same = digest == actual_digest;
        (digest, same)
    }) {
        Some((expected_digest, true)) => show_file_unchanged(path, &expected_digest),
        Some((expected_digest, false)) => show_file_changed(path, &actual_digest, &expected_digest),
        None => save_digest(path, &digest_file_path, &actual_digest),
    }
}

fn save_digest(path: &str, digest_path: &str, digest: &str) -> Result<(), DgStoreError> {
    fs::write(&digest_path, &digest).map_err(DgStoreError::Digest)?;

    println!(
        "{} {} {} {}",
        paint(Green, "✓"),
        paint(Cyan, format!("{:.7}", digest)),
        path,
        paint(Yellow, format!("(stored digest to {})", digest_path))
    );

    Ok(())
}

fn show_file_unchanged(path: &str, digest: &str) -> Result<(), DgStoreError> {
    println!(
        "{} {} {}",
        paint(Green, "✓"),
        paint(Green, format!("{:.7}", digest)),
        path
    );

    Ok(())
}

fn show_file_changed(path: &str, digest: &str, original_digest: &str) -> Result<(), DgStoreError> {
    println!(
        "{} {} {} {}",
        paint(Red, "✗"),
        paint(Red, format!("{:.7}", digest)),
        path,
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

fn read_digest_file(path: &str) -> Result<Option<String>, DgStoreError> {
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
