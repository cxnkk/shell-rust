[![progress-banner](https://backend.codecrafters.io/progress/shell/dcd630b4-8fd0-4f91-8610-90ee8b5dbbbc)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own Shell" Challenge](https://app.codecrafters.io/courses/shell/overview).

In this challenge, you'll build your own POSIX compliant shell that's capable of
interpreting shell commands, running external programs and builtin commands like
cd, pwd, echo and more. Along the way, you'll learn about shell command parsing,
REPLs, builtin commands, and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

Rust Shell (rsh)

A POSIX-compliant shell implementation written in Rust. This project explores the intricacies of process management, standard stream manipulation, and terminal raw mode handling to build a functional command-line interface from scratch.
🚀 Features

1. Core Functionality

   REPL (Read-Eval-Print Loop): Custom input handling loop supporting standard shell interaction.

   External Commands: Executes system binaries found in the $PATH environment variable.

   Input Parsing: robust handling of single quotes ('), double quotes ("), and backslash escaping.

2. Built-in Commands

Hand-rolled implementations of standard shell built-ins:

    cd: Change directory (supports absolute and relative paths).

    pwd: Print working directory.

    echo: Print arguments to stdout.

    type: Inspect command types (builtin vs. executable path).

    exit: Terminate the shell with a status code.

    history: View session command history.

3.  Advanced Process Management

    Pipelines (|): Full support for chaining commands (e.g., ls -l | grep ".rs" | wc -l).

        Implementation: Manually wires stdout of one child process to the stdin of the next using Stdio::piped() and Stdio::from_raw_fd.

    Input/Output Redirection: Supports stdout redirection (>) and stderr redirection (2>) (if implemented).

4.  Interactive UX

    Autocompletion: Press <TAB> to autocomplete executables in the system $PATH.

        Includes Longest Common Prefix (LCP) detection for multiple matches.

    History Navigation: Use ↑ (Up) and ↓ (Down) arrows to navigate previous commands.

        Implemented using an index pointer to avoid destructive memory operations.

5.  Persistence

    Session History: Commands are saved to a file defined by HISTFILE.

    Smart Appending: On exit, the shell intelligently appends only new commands to the history file, preserving existing data without truncation.

🧩 Technical Highlights

Pipeline Architecture

The pipeline engine is separated from the main event loop to ensure clean separation of concerns. It uses a "bucket brigade" strategy:

    Iterates through command segments split by |.

    Maintains a previous_command state.

    Wires the stdout of the previous child process into the stdin of the current process.

    Waits only for the final process in the chain to ensure the prompt returns correctly.

Memory Safe History

Unlike simple implementations that pop() commands off a stack, this shell uses a non-destructive index pointer pattern.

    Storage: A Vec<String> stores the session history.

    Navigation: A temporary history_index tracks the user's position during Up/Down navigation, resetting automatically when a new command is executed.

🧪 Testing

The project is validated against a rigorous suite of integration tests covering:

    Partial autocompletion logic.

    History file append/write modes.

    Complex quoting and spacing edge cases.
