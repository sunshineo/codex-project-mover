fn main() {
    if let Err(error) = codex_project_mover::run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
