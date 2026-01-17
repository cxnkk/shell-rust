mod quoting;
mod redirection;

use crate::quoting::parse_args;
use crate::redirection::parse_redirection;

use crossterm::{
    ExecutableCommand, cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio, exit};
use std::result::Result::Ok;
use std::{env, fs};
use std::{env::set_current_dir, io};

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

fn find_completions(prefix: &str) -> Vec<String> {
    if prefix.contains(" ") {
        return Vec::new();
    }

    let mut matches = Vec::new();

    let builtins = ["exit", "echo", "type", "pwd", "cd"];
    for builtin in builtins {
        if builtin.starts_with(prefix) {
            matches.push(builtin.to_string());
        }
    }

    if let Ok(path_var) = env::var("PATH") {
        for path in path_var.split(":") {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let filename = entry.file_name().into_string().unwrap_or_default();

                    if filename.starts_with(prefix) {
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.permissions().mode() & 0o111 != 0 {
                                matches.push(filename);
                            }
                        }
                    }
                }
            }
        }
    }

    matches.sort();
    matches.dedup();
    matches
}

fn main() {
    let mut stdout = io::stdout();

    loop {
        enable_raw_mode().unwrap();

        print!("$ ");
        stdout.flush().unwrap();

        let mut input_buffer = String::new();
        let mut cursor_position = 0;
        let mut tab_press_count = 0;

        loop {
            if let Event::Key(key) = event::read().unwrap() {
                if key.code != KeyCode::Tab {
                    tab_press_count = 0;
                }

                let mut execute_command = false;

                match key.code {
                    KeyCode::Tab => {
                        let matches = find_completions(&input_buffer);
                        tab_press_count += 1;

                        if matches.len() == 1 {
                            let completed = format!("{} ", matches[0]);
                            input_buffer = completed;
                            cursor_position = input_buffer.len();

                            stdout.execute(cursor::MoveToColumn(0)).unwrap();
                            stdout.execute(Clear(ClearType::CurrentLine)).unwrap();

                            print!("$ {}", input_buffer);
                            stdout.flush().unwrap();

                            tab_press_count = 0;
                        } else if matches.len() > 1 {
                            if tab_press_count == 1 {
                                print!("\x07");
                                stdout.flush().unwrap();
                            } else {
                                print!("\r\n");

                                let list = matches.join("  ");
                                print!("{}\r\n", list);

                                print!("$ {}", input_buffer);
                                stdout.flush().unwrap();
                            }
                        } else {
                            print!("\x07");
                            stdout.flush().unwrap();
                            tab_press_count = 0;
                        }
                    }
                    KeyCode::Enter => execute_command = true,
                    KeyCode::Backspace => {
                        if cursor_position > 0 {
                            input_buffer.remove(cursor_position - 1);
                            cursor_position -= 1;

                            stdout.execute(cursor::MoveToColumn(0)).unwrap();
                            stdout.execute(Clear(ClearType::CurrentLine)).unwrap();

                            print!("$ {}", input_buffer);

                            let new_pos = (cursor_position + 2) as u16;
                            stdout.execute(cursor::MoveToColumn(new_pos)).unwrap();

                            stdout.flush().unwrap();
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        println!("^C");
                        input_buffer.clear();
                        break;
                    }
                    KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        disable_raw_mode().unwrap();
                        exit(0);
                    }
                    KeyCode::Char(c) => {
                        if c == '\n' || (c == 'j' && key.modifiers.contains(KeyModifiers::CONTROL))
                        {
                            execute_command = true;
                        } else if !key.modifiers.contains(KeyModifiers::CONTROL)
                            && !key.modifiers.contains(KeyModifiers::ALT)
                        {
                            input_buffer.insert(cursor_position, c);
                            cursor_position += 1;
                            print!("{}", c);
                            stdout.flush().unwrap();
                        }
                    }
                    _ => {}
                }

                if execute_command {
                    disable_raw_mode().unwrap();
                    println!();

                    let input = input_buffer.trim();

                    if !input.is_empty() {
                        let parts = parse_args(input);

                        if !parts.is_empty() {
                            match Cmd::parse(&parts[0]) {
                                Cmd::Exit => exit(0),
                                Cmd::Echo => {
                                    let mut args = parts[1..].to_vec();

                                    let (stdout_opt, _stderr_opt) = parse_redirection(&mut args);

                                    let output_text = args.join(" ");

                                    match stdout_opt {
                                        Some(mut file) => {
                                            writeln!(file, "{}", output_text).unwrap()
                                        }
                                        None => {
                                            println!("{}", output_text);
                                        }
                                    }
                                }
                                Cmd::Type => match parts[1].as_str() {
                                    "exit" | "echo" | "type" | "pwd" | "cd" => {
                                        println!("{} is a shell builtin", parts[1])
                                    }
                                    _ => {
                                        let mut found = false;

                                        if let Ok(path_var) = env::var("PATH") {
                                            for path in path_var.split(':') {
                                                let full_path = format!("{}/{}", path, parts[1]);

                                                if Path::new(&full_path).exists() {
                                                    if let Ok(metadata) = fs::metadata(&full_path) {
                                                        if metadata.permissions().mode() & 0o111
                                                            != 0
                                                        {
                                                            println!(
                                                                "{} is {}",
                                                                parts[1], full_path
                                                            );
                                                            found = true;
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if !found {
                                            println!("{}: not found", parts[1]);
                                        }
                                    }
                                },
                                Cmd::Run => {
                                    let program_name = &parts[0];
                                    let mut args = parts[1..].to_vec();

                                    let (stdout_opt, stderr_opt) = parse_redirection(&mut args);

                                    let stdout_dest = match stdout_opt {
                                        Some(f) => Stdio::from(f),
                                        None => Stdio::inherit(),
                                    };

                                    let stderr_dest = match stderr_opt {
                                        Some(f) => Stdio::from(f),
                                        None => Stdio::inherit(),
                                    };

                                    match Command::new(program_name)
                                        .args(args)
                                        .stdout(stdout_dest)
                                        .stderr(stderr_dest)
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
                                            set_current_dir(&parts[1])
                                                .expect("Failed changing directory")
                                        } else {
                                            println!("cd: {}: No such file or directory", parts[1])
                                        }
                                    }
                                },
                            }
                        }
                    }

                    break;
                }
            }
        }
    }
}
