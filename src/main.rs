extern crate argparse;

use argparse::{ArgumentParser, Store, StoreTrue};
use error_stack::{IntoReport, Report, Result, ResultExt};
use std::{error::Error, fmt, fs};

#[derive(Debug)]
enum RenameError {
    InvalidInput(String),
    Other,
}

impl fmt::Display for RenameError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("Credit card error: Could not retrieve credit card.")
    }
}

impl Error for RenameError {}

fn iterate_dir(
    path: &str,
    extension: &String,
    dry_run: &bool,
    verbose: &bool,
) -> Result<(), RenameError> {
    let paths = fs::read_dir(path)
        .report()
        .change_context(RenameError::Other)
        .attach_printable(format!("Cannot read directory for {path}"))?;

    for entry in paths {
        let entry = entry
            .report()
            .change_context(RenameError::Other)
            .attach_printable(format!("Cannot get entry for {path}"))?;
        let path = entry.path();
        let data = entry
            .metadata()
            .report()
            .change_context(RenameError::Other)
            .attach_printable(format!("Cannot get metadata for {}", path.display()))?;
        if data.is_file() {
            if let Some(ex) = path.extension() {
                let ex_str = ex.to_str().ok_or_else(|| {
                    let msg = format!("Cannot get extension for file {}.", path.display());
                    Report::new(RenameError::InvalidInput(msg.clone()))
                        .attach_printable(msg.clone())
                })?;
                if ex_str == extension {
                    let file_name_os = path.file_name().ok_or_else(|| {
                        Report::new(RenameError::Other).attach_printable(format!(
                            "Cannot get file name for {}.",
                            path.display()
                        ))
                    })?;
                    let file_name = file_name_os.to_str().ok_or_else(|| {
                        Report::new(RenameError::Other).attach_printable(format!(
                            "Cannot get file name for {}.",
                            path.display()
                        ))
                    })?;
                    if file_name.contains(char::is_whitespace) {
                        let mut new_file_path = path.clone();
                        new_file_path.pop();
                        let new_filename = file_name.replace(" ", "_");
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
                            fs::rename(&path, &new_file_path)
                                .report()
                                .change_context(RenameError::Other)
                                .attach_printable(format!(
                                    "Cannot rename file from {} to {}",
                                    path.display(),
                                    new_file_path.display()
                                ))?;
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
        } else {
            if *verbose {
                println!("Directory: {}", path.display());
            }
            let path_str = path.to_str().ok_or_else(|| {
                Report::new(RenameError::Other)
                    .attach_printable(format!("Cannot get string from {}.", path.display()))
            })?;
            iterate_dir(path_str, extension, dry_run, verbose)?;
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
            "Recursively repalce whitespaces in filesnames matching the extension by underscores.",
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

    let result = iterate_dir(&path, &extension, &dry_run, &true);
    match result {
        Ok(()) => {
            println!("\nDone");
        }
        Err(err) => {
            match err.current_context() {
                RenameError::InvalidInput(msg) => println!("\n{msg}"),
                RenameError::Other => println!("\nSomething went wrong! Try again!"),
            }

            log::error!("\n{err:?}");
        }
    }
}
