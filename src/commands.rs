//Command handling logic
pub fn handle_command(command: String) {
    if command.trim() == "exit" {
        std::process::exit(0);
    }

    println!("{}: command not found", command.trim());
}