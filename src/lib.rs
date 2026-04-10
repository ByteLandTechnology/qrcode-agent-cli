pub mod context;
pub mod help;

use clap::{Args, CommandFactory, Parser, Subcommand};
use image::Luma;
use qrcode::{QrCode, render::unicode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::context::{
    ActiveContext, DEFAULT_IMAGE_SIZE, clear_active_context, default_output_path,
    load_active_context, runtime_env_overrides, runtime_paths, save_active_context,
};
use crate::help::structured_help;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    Text,
    Image,
}

impl RenderMode {
    fn parse(input: &str) -> Result<Self, String> {
        match input.to_ascii_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "image" | "png" => Ok(Self::Image),
            _ => Err(format!(
                "Unsupported render mode `{input}`. Use `text` or `image`."
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum OutputFormat {
    Yaml,
    Json,
    Toml,
}

impl OutputFormat {
    fn parse(input: &str) -> Result<Self, String> {
        match input.to_ascii_lowercase().as_str() {
            "yaml" => Ok(Self::Yaml),
            "json" => Ok(Self::Json),
            "toml" => Ok(Self::Toml),
            _ => Err(format!(
                "Unsupported format `{input}`. Use `yaml`, `json`, or `toml`."
            )),
        }
    }

    fn fallback(input: &str) -> Self {
        Self::parse(input).unwrap_or(Self::Yaml)
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Toml => "toml",
        }
    }

    fn serialize<T: Serialize>(self, value: &T) -> Result<String, AppError> {
        let rendered = match self {
            Self::Yaml => serde_yaml::to_string(value).map_err(|error| {
                format!("Failed to serialize output as {}: {error}", self.as_str())
            }),
            Self::Json => serde_json::to_string_pretty(value).map_err(|error| {
                format!("Failed to serialize output as {}: {error}", self.as_str())
            }),
            Self::Toml => toml::to_string_pretty(value).map_err(|error| {
                format!("Failed to serialize output as {}: {error}", self.as_str())
            }),
        }
        .map_err(|message| AppError::new("serialization_failed", message, self, 1))?;

        Ok(with_trailing_newline(rendered))
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "qrcode-agent-cli",
    version,
    about = "Generate QR codes as terminal text or PNG images",
    disable_help_subcommand = true,
    disable_colored_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run(RunArgs),
    Paths(StructuredArgs),
    Context(ContextArgs),
    Help(HelpArgs),
}

#[derive(Args, Debug)]
struct RunArgs {
    #[arg(value_name = "CONTENT")]
    content: Option<String>,
    #[arg(long, value_name = "MODE")]
    render: Option<String>,
    #[arg(long, short = 'o', value_name = "FILE")]
    output: Option<PathBuf>,
    #[arg(long, value_name = "PIXELS")]
    size: Option<u32>,
    #[arg(long, default_value = "yaml", value_name = "FORMAT")]
    format: String,
}

#[derive(Args, Debug)]
struct StructuredArgs {
    #[arg(long, default_value = "yaml", value_name = "FORMAT")]
    format: String,
}

#[derive(Args, Debug)]
#[command(disable_help_subcommand = true)]
struct ContextArgs {
    #[command(subcommand)]
    command: Option<ContextCommand>,
}

#[derive(Subcommand, Debug)]
enum ContextCommand {
    Show(StructuredArgs),
    Set(ContextSetArgs),
    Clear(StructuredArgs),
}

#[derive(Args, Debug)]
struct ContextSetArgs {
    #[arg(long, value_name = "MODE")]
    render: Option<String>,
    #[arg(long, short = 'o', value_name = "FILE")]
    output: Option<PathBuf>,
    #[arg(long, value_name = "PIXELS")]
    size: Option<u32>,
    #[arg(long, default_value = "yaml", value_name = "FORMAT")]
    format: String,
}

#[derive(Args, Debug)]
struct HelpArgs {
    #[arg(value_name = "COMMAND")]
    command_path: Vec<String>,
    #[arg(long, default_value = "yaml", value_name = "FORMAT")]
    format: String,
}

#[derive(Debug)]
struct AppError {
    code: String,
    message: String,
    details: Vec<String>,
    format: OutputFormat,
    exit_code: i32,
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
    code: &'a str,
    message: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    details: &'a Vec<String>,
}

#[derive(Serialize)]
struct PathsResponse {
    command: &'static str,
    runtime_directories: context::RuntimePaths,
    user_scoped_defaults: bool,
    env_overrides: context::RuntimeEnvOverrides,
    active_context_file_exists: bool,
}

#[derive(Serialize)]
struct ContextResponse {
    command: &'static str,
    active_context: ActiveContext,
    runtime_directories: context::RuntimePaths,
    source: ContextSource,
}

#[derive(Serialize)]
struct ContextSource {
    path: String,
    exists: bool,
}

#[derive(Serialize)]
struct ContextClearResponse {
    command: &'static str,
    cleared: bool,
    runtime_directories: context::RuntimePaths,
}

#[derive(Serialize)]
struct ImageRunResponse {
    command: &'static str,
    status: &'static str,
    render: RenderMode,
    content: String,
    image_path: String,
    image_size: u32,
    output_format: String,
    effective_context: EffectiveContextView,
}

#[derive(Serialize)]
struct EffectiveContextView {
    persisted_defaults: ActiveContext,
    invocation_overrides: InvocationOverrides,
    resolved: ActiveContext,
}

#[derive(Serialize)]
struct InvocationOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    render: Option<RenderMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_size: Option<u32>,
}

struct ResolvedRun {
    format: OutputFormat,
    content: String,
    render: RenderMode,
    output: Option<PathBuf>,
    image_size: u32,
    effective_context: EffectiveContextView,
}

impl AppError {
    fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        format: OutputFormat,
        exit_code: i32,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Vec::new(),
            format,
            exit_code,
        }
    }

    fn render(&self) -> String {
        let body = ErrorResponse {
            code: &self.code,
            message: &self.message,
            details: &self.details,
        };

        let rendered = match self.format {
            OutputFormat::Yaml => serde_yaml::to_string(&body).map_err(|error| error.to_string()),
            OutputFormat::Json => {
                serde_json::to_string_pretty(&body).map_err(|error| error.to_string())
            }
            OutputFormat::Toml => toml::to_string_pretty(&body).map_err(|error| error.to_string()),
        };

        match rendered {
            Ok(text) => with_trailing_newline(text),
            Err(_) => {
                with_trailing_newline(format!("code: {}\nmessage: {}", self.code, self.message))
            }
        }
    }

    fn print(&self) {
        let _ = io::stderr().write_all(self.render().as_bytes());
    }
}

pub fn run() -> i32 {
    match Cli::try_parse() {
        Ok(cli) => dispatch(cli),
        Err(error) => {
            let exit_code = if error.use_stderr() { 2 } else { 0 };
            let _ = error.print();
            exit_code
        }
    }
}

fn dispatch(cli: Cli) -> i32 {
    match cli.command {
        None => print_help_and_exit(&[]),
        Some(Commands::Run(args)) => finish(handle_run(args)),
        Some(Commands::Paths(args)) => finish(handle_paths(args)),
        Some(Commands::Context(args)) => match args.command {
            None => print_help_and_exit(&["context".to_owned()]),
            Some(ContextCommand::Show(args)) => finish(handle_context_show(args)),
            Some(ContextCommand::Set(args)) => finish(handle_context_set(args)),
            Some(ContextCommand::Clear(args)) => finish(handle_context_clear(args)),
        },
        Some(Commands::Help(args)) => finish(handle_help(args)),
    }
}

fn handle_run(args: RunArgs) -> Result<String, AppError> {
    let resolved = resolve_run(args)?;
    let qr_code = QrCode::new(resolved.content.as_bytes()).map_err(|error| {
        AppError::new(
            "qrcode_generation_failed",
            format!("Failed to generate a QR code: {error}"),
            resolved.format,
            1,
        )
    })?;

    match resolved.render {
        RenderMode::Text => {
            let text = qr_code
                .render::<unicode::Dense1x2>()
                .quiet_zone(true)
                .build();
            Ok(with_trailing_newline(text))
        }
        RenderMode::Image => {
            let output = resolved.output.clone().ok_or_else(|| {
                AppError::new(
                    "missing_output",
                    "Image mode requires an output path",
                    resolved.format,
                    1,
                )
            })?;
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    AppError::new(
                        "output_directory_creation_failed",
                        format!(
                            "Failed to create output directory `{}`: {error}",
                            parent.display()
                        ),
                        resolved.format,
                        1,
                    )
                })?;
            }

            let image = qr_code
                .render::<Luma<u8>>()
                .min_dimensions(resolved.image_size, resolved.image_size)
                .build();
            image.save(&output).map_err(|error| {
                AppError::new(
                    "image_write_failed",
                    format!("Failed to write `{}`: {error}", output.display()),
                    resolved.format,
                    1,
                )
            })?;

            let response = ImageRunResponse {
                command: "run",
                status: "ok",
                render: RenderMode::Image,
                content: resolved.content,
                image_path: display_path(&output),
                image_size: resolved.image_size,
                output_format: resolved.format.as_str().to_owned(),
                effective_context: resolved.effective_context,
            };
            resolved.format.serialize(&response)
        }
    }
}

fn handle_paths(args: StructuredArgs) -> Result<String, AppError> {
    let format = parse_output_format(&args.format)?;
    let paths = runtime_paths().map_err(|error| {
        AppError::new("runtime_paths_unavailable", error.to_string(), format, 1)
    })?;
    let response = PathsResponse {
        command: "paths",
        active_context_file_exists: Path::new(&paths.context_file).exists(),
        runtime_directories: paths,
        user_scoped_defaults: true,
        env_overrides: runtime_env_overrides(),
    };
    format.serialize(&response)
}

fn handle_context_show(args: StructuredArgs) -> Result<String, AppError> {
    let format = parse_output_format(&args.format)?;
    let paths = runtime_paths().map_err(|error| {
        AppError::new("runtime_paths_unavailable", error.to_string(), format, 1)
    })?;
    let response = ContextResponse {
        command: "context show",
        active_context: load_active_context()
            .map_err(|error| AppError::new("context_load_failed", error.to_string(), format, 1))?,
        source: ContextSource {
            exists: Path::new(&paths.context_file).exists(),
            path: paths.context_file.clone(),
        },
        runtime_directories: paths,
    };
    format.serialize(&response)
}

fn handle_context_set(args: ContextSetArgs) -> Result<String, AppError> {
    let format = parse_output_format(&args.format)?;
    if args.render.is_none() && args.output.is_none() && args.size.is_none() {
        return Err(AppError::new(
            "missing_context_changes",
            "Provide at least one of `--render`, `--output`, or `--size`.",
            format,
            1,
        ));
    }

    let mut active_context = load_active_context()
        .map_err(|error| AppError::new("context_load_failed", error.to_string(), format, 1))?;
    if let Some(render) = args.render.as_deref() {
        active_context.default_render = Some(parse_render_mode(render, format)?);
    }
    if let Some(output) = args.output {
        active_context.default_output = Some(display_path(&output));
    }
    if let Some(size) = args.size {
        if size == 0 {
            return Err(AppError::new(
                "invalid_size",
                "Image size must be greater than zero.",
                format,
                1,
            ));
        }
        active_context.default_image_size = Some(size);
    }

    save_active_context(&active_context)
        .map_err(|error| AppError::new("context_save_failed", error.to_string(), format, 1))?;

    let paths = runtime_paths().map_err(|error| {
        AppError::new("runtime_paths_unavailable", error.to_string(), format, 1)
    })?;
    let response = ContextResponse {
        command: "context set",
        active_context,
        source: ContextSource {
            exists: true,
            path: paths.context_file.clone(),
        },
        runtime_directories: paths,
    };
    format.serialize(&response)
}

fn handle_context_clear(args: StructuredArgs) -> Result<String, AppError> {
    let format = parse_output_format(&args.format)?;
    let cleared = clear_active_context()
        .map_err(|error| AppError::new("context_clear_failed", error.to_string(), format, 1))?;
    let paths = runtime_paths().map_err(|error| {
        AppError::new("runtime_paths_unavailable", error.to_string(), format, 1)
    })?;
    let response = ContextClearResponse {
        command: "context clear",
        cleared,
        runtime_directories: paths,
    };
    format.serialize(&response)
}

fn handle_help(args: HelpArgs) -> Result<String, AppError> {
    let format = parse_output_format(&args.format)?;
    let paths = runtime_paths().map_err(|error| {
        AppError::new("runtime_paths_unavailable", error.to_string(), format, 1)
    })?;
    let help_doc = structured_help(&args.command_path, &paths).ok_or_else(|| {
        AppError::new(
            "unknown_command_path",
            format!(
                "Unknown command path `{}`.",
                if args.command_path.is_empty() {
                    "<root>".to_owned()
                } else {
                    args.command_path.join(" ")
                }
            ),
            format,
            1,
        )
    })?;
    format.serialize(&help_doc)
}

fn resolve_run(args: RunArgs) -> Result<ResolvedRun, AppError> {
    let format = parse_output_format(&args.format)?;
    let persisted = load_active_context()
        .map_err(|error| AppError::new("context_load_failed", error.to_string(), format, 1))?;
    let content = args.content.ok_or_else(|| {
        AppError::new(
            "missing_content",
            "Provide inline content to encode, for example: `qrcode-agent-cli run \"hello\"`.",
            format,
            1,
        )
    })?;

    let render = match args.render.as_deref() {
        Some(render) => parse_render_mode(render, format)?,
        None => persisted.default_render.unwrap_or(RenderMode::Text),
    };
    let image_size = match args.size {
        Some(0) => {
            return Err(AppError::new(
                "invalid_size",
                "Image size must be greater than zero.",
                format,
                1,
            ));
        }
        Some(size) => size,
        None => persisted.default_image_size.unwrap_or(DEFAULT_IMAGE_SIZE),
    };
    let output_override = args.output.as_ref().map(|path| display_path(path));
    let effective_output = match render {
        RenderMode::Text => {
            if args.output.is_some() {
                return Err(AppError::new(
                    "output_not_supported_for_text",
                    "Do not pass `--output` when `--render text` is selected.",
                    format,
                    1,
                ));
            }
            None
        }
        RenderMode::Image => Some(
            args.output
                .clone()
                .or_else(|| persisted.default_output.clone().map(PathBuf::from))
                .unwrap_or(default_output_path().map_err(|error| {
                    AppError::new("default_output_unavailable", error.to_string(), format, 1)
                })?),
        ),
    };

    Ok(ResolvedRun {
        format,
        content,
        render,
        output: effective_output.clone(),
        image_size,
        effective_context: EffectiveContextView {
            persisted_defaults: persisted.clone(),
            invocation_overrides: InvocationOverrides {
                render: args
                    .render
                    .as_deref()
                    .map(|render| RenderMode::parse(render).unwrap_or(RenderMode::Text)),
                output: output_override,
                image_size: args.size,
            },
            resolved: ActiveContext {
                default_render: Some(render),
                default_output: effective_output.as_ref().map(|path| display_path(path)),
                default_image_size: Some(image_size),
            },
        },
    })
}

fn parse_output_format(input: &str) -> Result<OutputFormat, AppError> {
    OutputFormat::parse(input).map_err(|message| {
        AppError::new(
            "invalid_output_format",
            message,
            OutputFormat::fallback(input),
            1,
        )
    })
}

fn parse_render_mode(input: &str, format: OutputFormat) -> Result<RenderMode, AppError> {
    RenderMode::parse(input)
        .map_err(|message| AppError::new("invalid_render_mode", message, format, 1))
}

fn print_help_and_exit(path: &[String]) -> i32 {
    match command_for_path(path) {
        Some(mut command) => {
            if command.write_long_help(&mut io::stdout()).is_ok() {
                let _ = writeln!(io::stdout());
            }
            0
        }
        None => {
            AppError::new(
                "internal_help_error",
                "Unable to render the requested help command.",
                OutputFormat::Yaml,
                1,
            )
            .print();
            1
        }
    }
}

fn command_for_path(path: &[String]) -> Option<clap::Command> {
    let mut current = Cli::command();
    for segment in path {
        let next = current
            .get_subcommands()
            .find(|command| command.get_name() == segment.as_str())
            .cloned()?;
        current = next;
    }
    Some(current)
}

fn finish(result: Result<String, AppError>) -> i32 {
    match result {
        Ok(output) => {
            print!("{output}");
            0
        }
        Err(error) => {
            error.print();
            error.exit_code
        }
    }
}

fn with_trailing_newline(mut value: String) -> String {
    if !value.ends_with('\n') {
        value.push('\n');
    }
    value
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}
