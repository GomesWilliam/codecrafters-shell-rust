// Enum of built-in commands and their handlers
use std::{env, path::{MAIN_SEPARATOR, PathBuf}};

enum BuiltInCommand {
    Exit,
    Echo,
    Type,
    Unknown(String),
}

impl BuiltInCommand {
    fn from_str(s: &str) -> BuiltInCommand {
        match s {
            "exit" => BuiltInCommand::Exit,
            "echo" => BuiltInCommand::Echo,
            "type" => BuiltInCommand::Type,
            other  => BuiltInCommand::Unknown(other.to_string()),
        }
    }
}

pub fn handle_command(command: String, args: Vec<String>) {
    match BuiltInCommand::from_str(command.trim()) {
        BuiltInCommand::Exit           => std::process::exit(0),
        BuiltInCommand::Echo           => println!("{}", args.join(" ")),
        BuiltInCommand::Type           => type_command(args),
        BuiltInCommand::Unknown(cmd)   => println!("{}: command not found", cmd),
    }
}

pub fn type_command(args: Vec<String>) {
    if args.is_empty() {
        println!("type: missing operand");
        return;
    }

    //Compare the arguments to the list of built-in commands and print out which ones are built-in
    for arg in args {
        match BuiltInCommand::from_str(arg.trim()) {
            BuiltInCommand::Unknown(_) => type_non_builtin(&arg),
            _                          => println!("{} is a shell builtin", arg),
        }
    }
}

pub fn parse_command(input: String) -> (String, Vec<String>) {
    let parts: Vec<String> = input.trim().split_whitespace().map(String::from).collect();
    if parts.is_empty() {
        return (String::new(), Vec::new());
    }

    let command: String  = parts[0].clone();
    let args: Vec<String> = parts[1..].to_vec();

    (command, args)
}

pub fn type_non_builtin(name: &str) {
    let paths: Vec<PathBuf> = get_path_dirs();
    let mut found: Option<PathBuf> = None;

    for dir in &paths {
        if let Some(full_path) = find_executable(dir, name) {
            found = Some(full_path);
            break;
        }
    }

    if let Some(full_path) = found {
        println!("{} is {}", name, full_path.display());
    } else {
        println!("{}: not found", name);
    }
}

pub fn find_executable(dir: &PathBuf, name: &str) -> Option<PathBuf> {
    let full_path = dir.join(name);
    if  full_path.exists() {
        return Some(full_path);
    }

    None
}

pub fn get_path_dirs() -> Vec<PathBuf> {
    if let Ok(path) = env::var("PATH") {
        path
            .split(MAIN_SEPARATOR)
            .map(PathBuf::from)
            .collect()
    } else {
        Vec::new()
    }
}