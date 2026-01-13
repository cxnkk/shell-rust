use std::env::set_current_dir;
#[allow(unused_imports)]
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio, exit};
use std::result::Result::Ok;
use std::{env, fs};

enum Cmd {
    Exit,
    Echo,
    Type,
    Run,
    Pwd,
    Cd,
}

impl Cmd {
    fn parse(s: &str) -> Self {
        match s {
            "exit" => Cmd::Exit,
            "echo" => Cmd::Echo,
            "type" => Cmd::Type,
            "pwd" => Cmd::Pwd,
            "cd" => Cmd::Cd,
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

fn parse_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut escape_next = false;

    for c in input.chars() {
        if in_single_quotes {
            if c == '\'' {
                in_single_quotes = false;
            } else {
                current_arg.push(c);
            }
        } else if escape_next {
            escape_next = false;

            if in_double_quotes {
                match c {
                    '$' | '`' | '"' | '\\' | '\n' => {
                        current_arg.push(c);
                    }
                    _ => {
                        current_arg.push('\\');
                        current_arg.push(c);
                    }
                }
            } else {
                current_arg.push(c);
            }
        } else {
            match c {
                '\\' => escape_next = true,
                '\'' => {
                    if in_double_quotes {
                        current_arg.push(c);
                    } else {
                        in_single_quotes = true;
                    }
                }
                '"' => in_double_quotes = !in_double_quotes,
                ' ' | '\t' | '\n' | '\r' => {
                    if in_double_quotes {
                        current_arg.push(c);
                    } else if !current_arg.is_empty() {
                        args.push(current_arg);
                        current_arg = String::new();
                    }
                }
                _ => current_arg.push(c),
            }
        }
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    args
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let input = command.trim();
        let parts: Vec<String> = parse_args(input);

        match Cmd::parse(&parts[0]) {
            Cmd::Exit => exit(0),
            Cmd::Echo => {
                let mut args = parts[1..].to_vec();
                let mut output_file: Option<File> = None;

                if let Some(index) = args
                    .iter()
                    .position(|operator| operator == ">" || operator == "1>")
                {
                    if index + 1 < args.len() {
                        let filename = &args[index + 1];
                        if let Ok(file) = File::create(filename) {
                            output_file = Some(file)
                        }

                        args.remove(index + 1);
                        args.remove(index);
                    }
                }

                let output_text = args.join(" ");

                match output_file {
                    Some(mut file) => writeln!(file, "{}", output_text).unwrap(),
                    None => println!("{}", output_text),
                }
            }
            Cmd::Type => match parts[1].as_str() {
                "exit" | "echo" | "type" | "pwd" | "cd" => {
                    println!("{} is a shell builtin", parts[1])
                }
                _ => {
                    if let Some(path) = find_executable(&parts[1]) {
                        println!("{} is {}", parts[1], path);
                    } else {
                        println!("{}: not found", parts[1])
                    }
                }
            },
            Cmd::Run => {
                let program_name = &parts[0];
                let mut args = parts[1..].to_vec();
                let mut stdout_dest = Stdio::inherit();

                if let Some(index) = args
                    .iter()
                    .position(|operator| operator == ">" || operator == "1>")
                {
                    if index + 1 < args.len() {
                        let filename = &args[index + 1];

                        let file = File::create(filename).unwrap();

                        stdout_dest = Stdio::from(file);

                        args.remove(index + 1);
                        args.remove(index);
                    }
                }

                match Command::new(program_name)
                    .args(args)
                    .stdout(stdout_dest)
                    .spawn()
                {
                    Ok(mut child) => {
                        let _ = child.wait();
                    }
                    Err(_) => {
                        println!("{}: command not found", input)
                    }
                }
            }
            Cmd::Pwd => {
                let path = env::current_dir().expect("Not existing");
                println!("{}", path.display());
            }
            Cmd::Cd => match parts[1].as_str() {
                "~" => {
                    let home = env::home_dir().expect("No home dir found");
                    set_current_dir(home).expect("Failed changing directory")
                }
                _ => {
                    if Path::new(&parts[1]).exists() {
                        set_current_dir(&parts[1]).expect("Failed changing directory")
                    } else {
                        println!("cd: {}: No such file or directory", parts[1])
                    }
                }
            },
        }
    }
}
