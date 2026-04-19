#[allow(unused_imports)]
use std::io::{self, Write};
mod commands;

fn main() {
    // TODO: Uncomment the code below to pass the first stage

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        //Read user input and print it back to the console
        let mut command: String = String::new();
        io::stdin().read_line(&mut command).expect("Failed to read line");

        //Passes the command to the command handler
        commands::handle_command(command);
    }
}
