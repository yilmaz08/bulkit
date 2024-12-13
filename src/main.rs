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

    #[arg(help="Select editor (default: $EDITOR)", short, long)]
    editor: Option<String>
}

fn edit(paths: Vec<PathBuf>, editor: String) -> Vec<String> {
    let mut temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_path_buf();

    for i in 0..paths.len() {
        writeln!(temp_file, "{}", paths[i].display()).unwrap();
    }

    let mut command = Command::new(editor);
    command.arg(&temp_path);
    println!(">>> Running: {} {}", &command.get_program().to_str().unwrap(), &temp_path.display());
    command.status().expect("Failed to open editor");

    let content = fs::read_to_string(temp_path).unwrap();
    content.split("\n").map(|s| s.to_string()).collect::<Vec<String>>()
}

fn compare(original_paths: Vec<PathBuf>, new_paths: Vec<String>) -> (Vec<(PathBuf, PathBuf)>, Vec<PathBuf>) {
    let mut paths_move: Vec<(PathBuf, PathBuf)> = Vec::<(PathBuf, PathBuf)>::new();
    let mut paths_delete: Vec<PathBuf> = Vec::<PathBuf>::new();
    for i in 0..original_paths.len() {
        if new_paths.len() <= i {
            paths_delete.push(original_paths[i].clone());
            continue;
        }
        if new_paths[i].is_empty() {
            paths_delete.push(original_paths[i].clone());
            continue;
        }
        let path = PathBuf::from(new_paths[i].clone());
        if get_absolute_path(&path) != get_absolute_path(&original_paths[i]) {
            paths_move.push((original_paths[i].clone(), path));
        }
    }

    // Confirmation
    if paths_move.len() == 0 && paths_delete.len() == 0 {
        println!(">>> Nothing changed!");
        return (Vec::<(PathBuf, PathBuf)>::new(), Vec::<PathBuf>::new());
    }
    if paths_move.len() > 0 {
        println!(">>> Move:");
        for path in &paths_move {
            println!("\"{}\" -> \"{}\"", path.0.display(), path.1.display());
        }
    }
    if paths_delete.len() > 0 {
        println!(">>> Delete:");
        for path in &paths_delete {
            println!("\"{}\"", path.display());
        }
    }
    if !yes_no(">>> Are you sure?") {
        return (Vec::<(PathBuf, PathBuf)>::new(), Vec::<PathBuf>::new());
    }

    return (paths_move, paths_delete);
}

fn move_file(original: PathBuf, new: PathBuf) {
    if let Some(parent_dir) = new.parent() {
        fs::create_dir_all(parent_dir).unwrap();
    }
    fs::rename(original, new).unwrap();
}

fn delete_file(path: PathBuf) {
    if path.is_dir() {
        fs::remove_dir_all(path).unwrap();
    } else if path.is_file() {
        fs::remove_file(path).unwrap();
    } else {
        println!("Unknown file type");
    }
}

fn main() {
    let args = Args::parse();

    let editor = match args.editor {
        Some(val) => val,
        None => env::var("EDITOR").unwrap_or("nano".to_string())
    };

    let new_paths = edit(args.files.clone(), editor);
    let (paths_move, paths_delete) = compare(args.files, new_paths);

    for path in paths_move {
        move_file(path.0, path.1);
    }
    for path in paths_delete {
        delete_file(path);
    }
}
