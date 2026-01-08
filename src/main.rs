#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, exit};
use std::result::Result::Ok;
use std::{env, fs};

enum Cmd {
    Exit,
    Echo,
    Type,
    Run,
}

impl Cmd {
    fn parse(s: &str) -> Self {
        match s {
            "exit" => Cmd::Exit,
            "echo" => Cmd::Echo,
            "type" => Cmd::Type,
            _ => Cmd::Run,
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

        match Cmd::parse(parts[0]) {
            Cmd::Exit => exit(0),
            Cmd::Echo => {
                let msg = &parts[1..];
                println!("{}", msg.join(" "))
            }
            Cmd::Type => match parts[1] {
                "exit" | "echo" | "type" => println!("{} is a shell builtin", parts[1]),
                _ => {
                    if let Some(path) = find_executable(parts[1]) {
                        println!("{} is {}", parts[1], path);
                    } else {
                        println!("{}: not found", parts[1])
                    }
                }
            },
            Cmd::Run => {
                let program_name = parts[0];
                let args = &parts[1..];

                match Command::new(program_name).args(args).spawn() {
                    Ok(mut child) => {
                        let _ = child.wait();
                    }
                    Err(_) => {
                        println!("{}: command not found", input)
                    }
                }
            }
        }
    }
}
