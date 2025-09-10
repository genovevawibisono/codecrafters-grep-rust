use crate::regex::Regex;

use anyhow::{Ok, Result};
use std::path::{PathBuf};
use std::fs::{read_to_string, read_dir};

fn search_file(path: &PathBuf, regex: &Regex, print_file_path: bool) -> Result<Vec<String>> {
    return Ok(read_to_string(path)?.lines().filter(|line| regex.matches(&line)).
        map(|line| {
            if print_file_path {
                format!("{}:{}", path.display(), line)
            } else {
                line.to_string()
            }
        }
    ).collect());
}

fn search_directory(path: &PathBuf, regex: &Regex) -> Result<Vec<String>> {
    let mut results = vec![];

    for entry in read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            results.extend(search_directory(&path, regex)?);
        } else if path.is_file() {
            results.extend(search_file(&path, regex, true)?);
        }
    }

    return Ok(results);
}

pub fn search_files(paths: Vec<PathBuf>, regex: Regex, recursive: bool) -> Result<Vec<String>> {
    if paths.len() == 1 && paths[0].is_file() == true {
        return search_file(&paths[0], &regex, false);
    }
    let mut results = vec![];

    for path in paths.into_iter().filter(|p| p.exists()) {
        if path.is_dir() == true && recursive {
            results.extend(search_directory(&path, &regex)?);
        } else if path.is_file() {
            results.extend(search_file(&path, &regex, true)?);
        }
    }

    return Ok(results);
}


