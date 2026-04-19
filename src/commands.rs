// Enum of built-in commands and their handlers
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
            BuiltInCommand::Unknown(_) => println!("{}: not found", arg),
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