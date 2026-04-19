//Command handling logic
pub fn handle_command(command: String, args: Vec<String>) {
    if command.trim() == "exit" {
        std::process::exit(0);
    }
    else if command.trim() == "echo" {
        println!("{}", args.join(" "));
    }
    else {
        println!("{}: command not found", command.trim());
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