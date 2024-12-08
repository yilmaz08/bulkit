use std::env;
use std::io::Write;
use tempfile::NamedTempFile;
use clap::Parser;
use std::{fs, path::PathBuf};
use std::process::Command;
use path_clean::PathClean;

fn get_absolute_path(relative_path: &PathBuf) -> PathBuf {
    env::current_dir().expect("Failed to get current directory").join(relative_path).clean()
}

fn yes_no(prompt: &str) -> bool {
    print!("{} (y/N): ", prompt);
    std::io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read input");

    match input.trim().to_lowercase().as_str() {
        "y" | "Y" => true,  // Yes
        "n" | "N" | "" => false,  // No or default
        _ => yes_no(prompt) // Unknown
    }
}

#[derive(Parser, Debug, Clone)]
#[command(about, version)]
struct Args {
    #[arg(help="Files to rename/move")]
    files: Vec<PathBuf>,
}

fn edit(paths: Vec<PathBuf>, editor: String) -> Vec<PathBuf> {
    let mut temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_path_buf();

    for i in 0..paths.len() {
        writeln!(temp_file, "{}", paths[i].display()).unwrap();
    }

    Command::new(editor)
        .arg(&temp_path)
        .status()
        .expect("Failed to open editor");

    let content = fs::read_to_string(temp_path).unwrap();
    let lines = content.split("\n").filter(|s| !s.is_empty()).collect::<Vec<&str>>();

    let mut new_paths: Vec<PathBuf> = Vec::<PathBuf>::new();
    for line in lines {
        new_paths.push(PathBuf::from(line));
    }

    return new_paths;
}

fn compare(original_paths: Vec<PathBuf>, new_paths: Vec<PathBuf>) -> Vec<(PathBuf, PathBuf)> {
    if original_paths.len() != new_paths.len() {
        panic!("Original and new path list sizes do not match! (new: {}, original: {})", new_paths.len(), original_paths.len());
    }
    
    let mut filtered_paths: Vec<(PathBuf, PathBuf)> = Vec::<(PathBuf, PathBuf)>::new();
    for i in 0..new_paths.len() {
        if get_absolute_path(&new_paths[i]) != get_absolute_path(&original_paths[i]) {
            filtered_paths.push((original_paths[i].clone(), new_paths[i].clone()));
            println!("\"{}\" -> \"{}\"", original_paths[i].clone().display(), new_paths[i].clone().display());
        }
    }

    if filtered_paths.len() > 0 && yes_no("Do you accept?") == false {
        return Vec::<(PathBuf, PathBuf)>::new();
    }

    return filtered_paths;
}

fn move_file(original: PathBuf, new: PathBuf) {
    if let Some(parent_dir) = new.parent() {
        fs::create_dir_all(parent_dir).unwrap();
    }
    fs::rename(original, new).unwrap();
}

fn main() {
    let args = Args::parse();

    let editor = env::var("EDITOR").unwrap_or("nano".to_string());

    let new_paths = edit(args.files.clone(), editor);
    let filtered_paths = compare(args.files, new_paths);

    for path_tuple in filtered_paths {
        move_file(path_tuple.0, path_tuple.1);
    }
}
