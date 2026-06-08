fn main() {
    if let Err(failure) = codex_project_mover::run() {
        if failure.json {
            codex_project_mover::print_json_error(&failure.error);
        } else {
            eprintln!("error: {}", failure.error);
        }
        std::process::exit(failure.error.exit_code().code());
    }
}
