#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::exit;
use std::{env, fs};

enum Command {
    Exit,
    Echo,
    Type,
    CommandNotFound,
}

impl Command {
    fn parse(s: &str) -> Self {
        match s {
            "exit" => Command::Exit,
            "echo" => Command::Echo,
            "type" => Command::Type,
            _ => Command::CommandNotFound,
        }
    }
}

fn find_executable(command_name: &str) -> Option<String> {
    if let Ok(path_var) = env::var("PATH") {
        for path in path_var.split(":") {
            let full_path = format!("{}/{}", path, command_name);
            if Path::new(&full_path).exists() {
                if let Ok(metadata) = fs::metadata(&full_path) {
                    let mode = metadata.permissions().mode();

                    if mode & 0o111 != 0 {
                        return Some(full_path);
                    }
                }
            }
        }
    }
    None
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let input = command.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();

        match Command::parse(parts[0]) {
            Command::Exit => exit(0),
            Command::Echo => {
                let msg = &parts[1..];
                println!("{}", msg.join(" "))
            }
            Command::Type => match parts[1] {
                "exit" | "echo" | "type" => println!("{} is a shell builtin", parts[1]),
                _ => {
                    if let Some(path) = find_executable(parts[1]) {
                        println!("{} is {}", parts[1], path);
                    } else {
                        println!("{}: not found", parts[1])
                    }
                }
            },
            Command::CommandNotFound => {
                println!("{}: command not found", input);
            }
        }
    }
}
