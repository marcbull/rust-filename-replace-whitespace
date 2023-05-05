extern crate argparse;

use anyhow::{anyhow, Context, Result};
use argparse::{ArgumentParser, Store, StoreTrue};
use std::{fmt, fs, fs::Metadata, path::PathBuf};

#[derive(Debug)]
enum RenameError {
    OsStrToStrFailed(String),
    PathToOsStrFailed(String),
    PathToStrFailed(String),
    RenameFailed(String),
    InvalidEntry(String),
    GetMetadataFailed(String),
    ReadDirFailed(String),
    UnknownMetadatyType(String),
}

impl fmt::Display for RenameError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("Could not rename files in directory")
    }
}

impl std::error::Error for RenameError {}

fn handle_file(
    path: &PathBuf,
    data: &Metadata,
    extension: &String,
    dry_run: &bool,
    verbose: &bool,
) -> Result<()> {
    if let Some(ex) = path.extension() {
        let ex = ex.to_str().ok_or_else(|| {
            let msg = format!("Cannot get extension for file {}", path.display());
            anyhow!(RenameError::OsStrToStrFailed(msg.clone())).context(msg)
        })?;
        if ex == extension {
            let file_name = path.file_name().ok_or_else(|| {
                let msg = format!("Cannot get file name for {}.", path.display());
                anyhow!(RenameError::PathToOsStrFailed(msg.clone())).context(msg)
            })?;
            let file_name = file_name.to_str().ok_or_else(|| {
                let msg = format!("Cannot get file name for {}.", path.display());
                anyhow!(RenameError::OsStrToStrFailed(msg.clone())).context(msg)
            })?;
            if file_name.contains(char::is_whitespace) {
                let mut new_file_path = path.clone();
                new_file_path.pop();
                let new_filename = file_name.replace(' ', "_");
                new_file_path.push(new_filename);
                let mut operation = "rename";
                if *dry_run {
                    operation = "would rename";
                }
                println!(
                    "File: matches {} {} from '{}' to '{}'",
                    extension,
                    operation,
                    path.display(),
                    new_file_path.display()
                );
                if !*dry_run {
                    fs::rename(path, &new_file_path).with_context(|| {
                        RenameError::RenameFailed(format!(
                            "Cannot rename file from {} to {}",
                            path.display(),
                            new_file_path.display()
                        ))
                    })?;
                }
            } else if *verbose {
                println!(
                    "{} file: {} length {}",
                    extension,
                    path.display(),
                    data.len()
                );
            }
        } else if *verbose {
            println!("File: {} length {}", path.display(), data.len());
        }
    }

    Ok(())
}

fn iterate_dir(path: &str, extension: &String, dry_run: &bool, verbose: &bool) -> Result<()> {
    let paths = fs::read_dir(path)
        .with_context(|| RenameError::ReadDirFailed(format!("Cannot read directory for {path}")))?;

    for entry in paths {
        let entry = entry
            .with_context(|| RenameError::InvalidEntry(format!("Cannot get entry for {path}")))?;
        let path = entry.path();
        let data = entry.metadata().with_context(|| {
            RenameError::GetMetadataFailed(format!("Cannot get metadata for {}", path.display()))
        })?;
        if data.is_file() {
            handle_file(&path, &data, extension, dry_run, verbose)?;
        } else if data.is_dir() {
            if *verbose {
                println!("Directory: {}", path.display());
            }
            let path = path.to_str().ok_or_else(|| {
                let msg = format!("Cannot get string from {}.", path.display());
                anyhow!(RenameError::PathToStrFailed(msg.clone())).context(msg)
            })?;
            iterate_dir(path, extension, dry_run, verbose)?;
        } else if data.is_symlink() {
            if *verbose {
                println!("Symlink: {} - not follwing", path.display());
            }
        } else {
            let msg = format!("Unkown metadaty type for for {}.", path.display());
            return Err(anyhow!(RenameError::UnknownMetadatyType(msg.clone())).context(msg));
        }
    }

    Ok(())
}

fn main() {
    let mut verbose = false;
    let mut dry_run = true;
    let mut extension = String::from("mkv");
    let mut path = String::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description(
            "Recursively repalce whitespaces in filesnames matching the extension by underscores. See: https://github.com/marcbull/rust-filename-replace-whitespace",
        );

        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Be verbose");

        ap.refer(&mut dry_run).add_option(
            &["-d", "--dry-run"],
            StoreTrue,
            "Do not rename only print what to do",
        );

        ap.refer(&mut extension).add_option(
            &["-e", "--extension"],
            Store,
            "Extension to search for",
        );

        ap.refer(&mut path)
            .add_argument("path", Store, "path to recursively search for files")
            .required();

        ap.parse_args_or_exit();
    }

    flexi_logger::Logger::try_with_env_or_str("trace")
        .unwrap()
        .start()
        .unwrap();

    match iterate_dir(&path, &extension, &dry_run, &verbose) {
        Ok(_) => {
            println!("\nDone");
        }
        Err(err) => {
            log::error!("\n{err:?}");
        }
    }
}
