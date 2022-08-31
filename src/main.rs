extern crate argparse;

use argparse::{ArgumentParser, Store, StoreTrue};
use std::fs;

fn iterate_dir(path: &str, extension: &String, dry_run: &bool, verbose: &bool) {
    let paths = fs::read_dir(path).unwrap();

    for entry in paths {
        let entry = entry.unwrap();
        let data = entry.metadata().unwrap();
        let path = entry.path();
        if data.is_file() {
            if let Some(ex) = path.extension() {
                if ex.to_str().unwrap() == extension {
                    if *verbose {
                        println!(
                            "{} file: {} length {}",
                            extension,
                            path.display(),
                            data.len()
                        );
                    }
                    let file_name = path.file_name().unwrap().to_str().unwrap();
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
                            "{} file: {} from '{}' to '{}'",
                            extension,
                            operation,
                            path.display(),
                            new_file_path.display()
                        );
                        if !*dry_run {
                            fs::rename(path, new_file_path).unwrap();
                        }
                    }
                }
            } else if *verbose {
                println!("File: {} length {}", path.display(), data.len());
            }
        } else {
            if *verbose {
                println!("Directory: {}", path.display());
            }
            iterate_dir(path.to_str().unwrap(), extension, dry_run, verbose);
        }
    }
}

fn main() {
    let mut verbose = false;
    let mut dry_run = false;
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

    iterate_dir(&path, &extension, &dry_run, &verbose);
}
