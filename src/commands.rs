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
    Cd,
    External,
}

impl BuiltInCommand {
    fn from_str(s: &str) -> BuiltInCommand {
        match s {
            "exit" => BuiltInCommand::Exit,
            "echo" => BuiltInCommand::Echo,
            "type" => BuiltInCommand::Type,
            "pwd"  => BuiltInCommand::Pwd,
            "cd"   => BuiltInCommand::Cd,
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
        BuiltInCommand::Echo           => echo_commnad(&args),
        BuiltInCommand::Type           => type_command(args),
        BuiltInCommand::Pwd            => pwd_command(),
        BuiltInCommand::Cd             => cd_command(&args),
        BuiltInCommand::External       => handle_non_builtin_command(command, &args),
    }
}

fn echo_commnad(args: &[String]){
    //Single quotes rules
    // Spaces are preserved within quotes.
    // Consecutive spaces are collapsed unless quoted.
    // Adjacent quoted strings 'hello' and 'world' are concatenated.
    //Empty quotes '' are ignored.

    // Quotes are already stripped by parse_command, so args are clean.
    // Skip tokens that ended up empty (e.g. from '' in input).
    let non_empty: Vec<&String> = args.iter().filter(|a| !a.is_empty()).collect();
    println!("{}", non_empty.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" "))
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
    // Tokenize while preserving whitespace inside single quotes.
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_single_quotes = false;

    for ch in input.chars() {
        match ch {
            '\'' => {
                // Toggle quote mode but do NOT push the quote char itself.
                // This way the token accumulates only the real content,
                // and every command (built-in or external) gets clean args.
                in_single_quotes = !in_single_quotes;
            }
            c if c.is_whitespace() && !in_single_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    let Some(command) = tokens.first().cloned() else {
        return (String::new(), Vec::new());
    };

    let args: Vec<String> = tokens.into_iter().skip(1).collect();

    (command, args)
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

fn cd_command(args: &[String]) {
    let Some(path_arg) = args.first() else {
        println!("cd: missing argument");
        return;
    };

    if args.len() > 1 {
        println!("cd: too many arguments");
        return;
    }

    let target = resolve_cd_path(path_arg);

    if let Err(err) = env::set_current_dir(&target) {
        let err_message = err.to_string();
        let clean_message = err_message
            .split(" (os error")
            .next()
            .unwrap_or(&err_message);
        println!("cd: {}: {}", path_arg, clean_message);
    }
}

fn resolve_cd_path(path_arg: &str) -> PathBuf {
    if path_arg == "~" {
        return env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(path_arg));
    }

    PathBuf::from(path_arg)
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