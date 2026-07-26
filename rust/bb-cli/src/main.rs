mod cli;

use std::io::{IsTerminal, stderr, stdin, stdout};
use std::process::ExitCode;

fn main() -> ExitCode {
    let request = match cli::parse_from(std::env::args_os()) {
        Ok(request) => request,
        Err(error) => {
            let use_stderr = error.use_stderr();
            let rendered = error.render().to_string();
            if use_stderr {
                eprint!("{rendered}");
            } else {
                print!("{rendered}");
            }
            return ExitCode::from(error.exit_code() as u8);
        }
    };

    let stdout = stdout();
    let stderr = stderr();
    let stdout_is_tty = stdout.is_terminal();
    let stderr_is_tty = stderr.is_terminal();
    let mut stdin = stdin().lock();
    let mut stdout = stdout.lock();
    let mut stderr = stderr.lock();
    ExitCode::from(bb_core::run(
        request,
        &mut stdin,
        &mut stdout,
        &mut stderr,
        stdout_is_tty,
        stderr_is_tty,
    ))
}
