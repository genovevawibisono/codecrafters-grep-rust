use std::io;
use std::path::PathBuf;
use std::process;

use clap::Parser;
use codecrafters_grep::file_search::search_files;
use codecrafters_grep::regex::Regex;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'E', required = true, help = "Regex pattern to search for")]
    regex: String,

    #[arg(short = 'r', long = "recursive", help = "Search recursively")]
    recursive: bool,

    #[arg(value_parser = clap::value_parser!(PathBuf))]
    paths: Vec<PathBuf>,
}

fn print_results(results: Vec<String>) {
    for result in results {
        println!("{result}");
    }
}

fn main() {
    let args = Args::parse();
    let regex = Regex::parse(args.regex.as_str());

    if !args.paths.is_empty() {
        match search_files(args.paths, regex, args.recursive) {
            Ok(res) if !res.is_empty() => {
                print_results(res);
                process::exit(0)
            }
            _ => {
                println!();
                process::exit(1)
            }
        }
    }

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();

    if regex.matches(&input_line) {
        println!("Match found: {}", input_line.trim());
        process::exit(0)
    } else {
        println!("Match not found: {}", input_line.trim());
        process::exit(1)
    }
}