// Command handling for built-ins and external executables.
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::os::unix::process::CommandExt;

enum BuiltInCommand {
    Exit,
    Echo,
    Type,
    Pwd,
    External,
}

impl BuiltInCommand {
    fn from_str(s: &str) -> BuiltInCommand {
        match s {
            "exit" => BuiltInCommand::Exit,
            "echo" => BuiltInCommand::Echo,
            "type" => BuiltInCommand::Type,
            "pwd"  => BuiltInCommand::Pwd,
            _ => BuiltInCommand::External,
        }
    }
}

pub fn handle_command(command: String, args: Vec<String>) {
    let command = command.trim();
    if command.is_empty() {
        return;
    }

    match BuiltInCommand::from_str(command) {
        BuiltInCommand::Exit           => std::process::exit(0),
        BuiltInCommand::Echo           => println!("{}", args.join(" ")),
        BuiltInCommand::Type           => type_command(args),
        BuiltInCommand::Pwd            => pwd_command(),
        BuiltInCommand::External       => handle_non_builtin_command(command, &args),
    }
}

fn handle_non_builtin_command(command: &str, args: &[String]) {
    let Some(found) = find_executable_in_dir(command) else {
        println!("{}: command not found", command);
        return;
    };

    run_external(&found, command, args);
}

pub fn type_command(args: Vec<String>) {
    if args.is_empty() {
        println!("type: missing operand");
        return;
    }

    //Compare the arguments to the list of built-in commands and print out which ones are built-in
    for arg in args {
        match BuiltInCommand::from_str(arg.trim()) {
            BuiltInCommand::External   => type_non_builtin(&arg),
            _                          => println!("{} is a shell builtin", arg),
        }
    }
}

pub fn parse_command(input: String) -> (String, Vec<String>) {
    let mut parts = input.split_whitespace();
    let Some(command) = parts.next() else {
        return (String::new(), Vec::new());
    };

    let args: Vec<String> = parts.map(String::from).collect();

    (command.to_string(), args)
}

pub fn type_non_builtin(name: &str) {
    let found: Option<PathBuf> = find_executable_in_dir(name);

    if let Some(full_path) = found {
        println!("{} is {}", name, full_path.display());
    } else {
        println!("{}: not found", name);
    }
}

fn pwd_command() {
    match env::current_dir() {
        Ok(current_dir) => println!("{}", current_dir.display()),
        Err(err) => eprintln!("pwd: {}", err),
    }
}

fn find_executable_in_dir(name: &str) -> Option<PathBuf> {
    get_path_dirs()
        .iter()
        .find_map(|dir| find_executable_in_path(dir, name))
}

#[cfg(unix)]
pub fn find_executable_in_path(dir: &Path, name: &str) -> Option<PathBuf> {
    let full_path = dir.join(name);
    if is_executable_file(&full_path) {
        return Some(full_path);
    }

    None
}

#[cfg(windows)]
pub fn find_executable_in_path(dir: &Path, name: &str) -> Option<PathBuf> {
    let pathexts = get_windows_pathexts();
    let has_ext = Path::new(name).extension().is_some();

    if has_ext {
        let full_path = dir.join(name);
        if is_executable_file(&full_path) && has_allowed_windows_extension(&full_path, &pathexts) {
            return Some(full_path);
        }
        return None;
    }

    for ext in pathexts {
        let full_path = dir.join(format!("{}{}", name, ext));
        if is_executable_file(&full_path) {
            return Some(full_path);
        }
    }

    None
}

fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };

    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(windows)]
fn get_windows_pathexts() -> Vec<String> {
    let raw = env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());

    raw.split(';')
        .map(str::trim)
        .filter(|ext| !ext.is_empty())
        .map(|ext| {
            let normalized = if ext.starts_with('.') {
                ext.to_string()
            } else {
                format!(".{}", ext)
            };
            normalized.to_ascii_uppercase()
        })
        .collect()
}

#[cfg(windows)]
fn has_allowed_windows_extension(path: &Path, pathexts: &[String]) -> bool {
    let Some(ext) = path.extension() else {
        return false;
    };

    let ext = format!(".{}", ext.to_string_lossy()).to_ascii_uppercase();
    pathexts.contains(&ext)
}

pub fn get_path_dirs() -> Vec<PathBuf> {
    if let Some(path) = env::var_os("PATH") {
        env::split_paths(&path).collect()
    } else {
        Vec::new()
    }
}

fn run_external(program: &Path, command_name: &str, args: &[String]) {
    let mut cmd = Command::new(program);

    #[cfg(unix)]
    {
        // Keep argv[0] as the original command token (e.g. "custom_exe_7296")
        // even when executing via a resolved absolute path.
        cmd.arg0(command_name);
    }

    let status = cmd.args(args).status();

    match status {
        Ok(exit_status) => {
            if !exit_status.success() {
                eprintln!("process exited with: {}", exit_status);
            }
        }
        Err(err) => {
            eprintln!("failed to execute {}: {}", program.display(), err);
        }
    }
}