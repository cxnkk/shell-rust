use std::fs::{File, OpenOptions};

pub fn parse_redirection(args: &mut Vec<String>) -> (Option<File>, Option<File>) {
    let mut stdout_file = None;
    let mut stderr_file = None;
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i].clone();

        if arg == ">" || arg == "1>" || arg == "2>" || arg == "2>>" || arg == ">>" || arg == "1>>" {
            if i + 1 >= args.len() {
                break;
            }

            let filename = &args[i + 1];

            match arg.as_str() {
                ">" | "1>" => {
                    let file = File::create(filename).unwrap();

                    stdout_file = Some(file)
                }
                ">>" | "1>>" => {
                    let file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(filename)
                        .expect("Cannot open file.");

                    stdout_file = Some(file)
                }
                "2>" => {
                    let file = File::create(filename).unwrap();

                    stderr_file = Some(file)
                }
                "2>>" => {
                    let file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(filename)
                        .expect("Cannot open file.");

                    stderr_file = Some(file)
                }
                _ => todo!(),
            }

            args.remove(i);
            args.remove(i);
        } else {
            i += 1;
        }
    }

    (stdout_file, stderr_file)
}
