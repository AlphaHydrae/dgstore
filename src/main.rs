use ansi_term::Colour::Cyan;
use ansi_term::Colour::Green;
use ansi_term::Colour::Red;
use glob::glob;
use glob::GlobError;
use glob::PatternError;
use sha2::{Digest, Sha512};
use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs::File;
use std::io;
use std::process;

enum DgStoreError {
    InvalidFilePath(OsString),
    Glob(GlobError),
    GlobPattern(String, PatternError),
}

impl fmt::Display for DgStoreError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
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
    if patterns.len() == 0 {
        println!("{}", Red.paint("At least one file argument is required"));
        process::exit(1);
    }

    match compute_and_store_digests(&patterns) {
        Ok(_) => {
            process::exit(0);
        }
        Err(error) => {
            println!("{}", Red.paint(format!("{}", error)));
            process::exit(2);
        }
    }
}

fn compute_and_store_digests(patterns: &Vec<String>) -> Result<(), DgStoreError> {
    for file in glob_files(patterns)? {
        match hash_file(&file) {
            Ok(_) => {}
            Err(error) => {
                println!(
                    "{}",
                    Red.paint(format!("Could not hash file {}: {}", file, error))
                );
                process::exit(1);
            }
        }
    }

    Ok(())
}

fn glob_files(patterns: &Vec<String>) -> Result<Vec<String>, DgStoreError> {
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

fn hash_file(path: &String) -> Result<(), io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Sha512::new();

    io::copy(&mut file, &mut hasher)?;

    let hex_digest = format!("{:x}", hasher.finalize());

    println!(
        "{} {} {}",
        Green.paint("✓"),
        Cyan.paint(format!("{:.7}", hex_digest)),
        path
    );
    Ok(())
}
