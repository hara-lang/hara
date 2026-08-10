#[path = "hara/cli.rs"]
mod cli;
#[path = "hara/repl.rs"]
mod repl;
#[path = "hara/terminal.rs"]
mod terminal;

// Debug evaluator frames are substantially larger than optimized frames. Keep
// the shipped runtime bounded at 8 MiB while giving developer builds enough
// room to bootstrap the portable library without hiding release regressions.
const CLI_STACK_SIZE: usize = if cfg!(debug_assertions) {
    64 * 1024 * 1024
} else {
    8 * 1024 * 1024
};

fn run_main() {
    let options = match cli::parse_options() {
        Ok(options) => options,
        Err(error) => cli::exit_error(&error, 2),
    };
    if let Err(error) = cli::run(options) {
        cli::exit_error(&error, cli::error_exit_code(&error));
    }
}

fn main() {
    let execution = std::thread::Builder::new()
        .name("hara-cli".into())
        .stack_size(CLI_STACK_SIZE)
        .spawn(run_main)
        .unwrap_or_else(|error| {
            eprintln!("hara: cannot start CLI thread: {error}");
            std::process::exit(2);
        });
    if let Err(panic) = execution.join() {
        std::panic::resume_unwind(panic);
    }
}
