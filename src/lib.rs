pub mod app_error;
pub mod backup;
pub mod cli;
pub mod commands;
pub mod discovery;
pub mod error;
pub mod git_worktree;
pub mod model;
pub mod output;
pub mod pathing;
pub mod process_guard;
pub mod project_copy;
pub mod scanner;
pub mod surfaces;
pub mod trash;
pub mod updater;

pub struct RunError {
    pub json: bool,
    pub error: app_error::AppError,
}

pub fn run() -> Result<(), RunError> {
    let cli = <cli::Cli as clap::Parser>::parse();
    let json = cli.json;
    let output_mode = output::OutputMode::from_json_flag(json);
    let result = match cli.command {
        cli::Command::Plan(args) => commands::plan::run(args, output_mode),
        cli::Command::Apply(args) => commands::apply::run(args, output_mode),
        cli::Command::Verify(args) => commands::verify::run(args, output_mode),
        cli::Command::Rollback(args) => commands::rollback::run(args, output_mode),
    };
    result.map_err(|error| RunError { json, error })
}

pub fn render_json_error(error: &app_error::AppError) -> serde_json::Value {
    let mut value = serde_json::json!({
        "status": "error",
        "exit_code": error.exit_code().code(),
        "error_kind": error.exit_code(),
        "message": error.to_string(),
    });
    if let Some(details) = error.details() {
        if let (Some(value_object), Some(details_object)) =
            (value.as_object_mut(), details.as_object())
        {
            for (key, detail_value) in details_object {
                value_object.insert(key.clone(), detail_value.clone());
            }
        } else if let Some(value_object) = value.as_object_mut() {
            value_object.insert("details".to_string(), details.clone());
        }
    }
    value
}

pub fn print_json_error(error: &app_error::AppError) {
    match serde_json::to_string(&render_json_error(error)) {
        Ok(json) => eprintln!("{json}"),
        Err(serialize_error) => eprintln!(
            "{{\"status\":\"error\",\"exit_code\":1,\"error_kind\":\"unexpected\",\"message\":\"failed to serialize error output: {}\"}}",
            serialize_error
        ),
    }
}
