#[allow(unused_imports)]
use std::io::{self, Write};
mod commands;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        //Read user input and print it back to the console
        let mut command: String = String::new();
        io::stdin().read_line(&mut command).expect("Failed to read line");

        //Passes the command to the command handler
        // Parse the command and its arguments
        let (command, args) = commands::parse_command(command);

        commands::handle_command(command, args);
    }
}
