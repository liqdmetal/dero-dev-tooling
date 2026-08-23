fn main() {
    std::process::exit(match tela_toolkit::cli::run() {
        std::process::ExitCode::SUCCESS => 0,
        _ => 1,
    });
}
