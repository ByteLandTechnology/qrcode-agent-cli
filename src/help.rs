use serde::Serialize;

use crate::context::{DEFAULT_IMAGE_SIZE, RuntimePaths};

#[derive(Debug, Clone, Serialize)]
pub struct HelpDoc {
    pub command_path: Vec<String>,
    pub summary: String,
    pub description: String,
    pub subcommands: Vec<String>,
    pub options: Vec<HelpOption>,
    pub output_formats: Vec<String>,
    pub runtime_directories: RuntimePaths,
    pub active_context: ActiveContextDoc,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HelpOption {
    pub name: String,
    pub value_type: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveContextDoc {
    pub persisted_location: String,
    pub supported_fields: Vec<String>,
    pub override_precedence: String,
}

pub fn structured_help(path: &[String], runtime_paths: &RuntimePaths) -> Option<HelpDoc> {
    let active_context = ActiveContextDoc {
        persisted_location: runtime_paths.context_file.clone(),
        supported_fields: vec![
            "default_render".to_owned(),
            "default_output".to_owned(),
            "default_image_size".to_owned(),
        ],
        override_precedence:
            "Explicit run flags win for the current invocation and never mutate persisted defaults"
                .to_owned(),
    };

    match path {
        [] => Some(HelpDoc {
            command_path: vec!["qrcode-agent-cli".to_owned()],
            summary: "Generate QR codes as terminal text or PNG images".to_owned(),
            description: "Use `run` for QR generation, `paths` to inspect runtime directories, `context` to persist default settings, and `help` for structured help output.".to_owned(),
            subcommands: vec![
                "run".to_owned(),
                "paths".to_owned(),
                "context".to_owned(),
                "help".to_owned(),
            ],
            options: Vec::new(),
            output_formats: format_list(),
            runtime_directories: runtime_paths.clone(),
            active_context,
            examples: vec![
                "qrcode-agent-cli run \"https://openai.com\" --render text".to_owned(),
                "qrcode-agent-cli run \"hello\" --render image --output ./hello.png".to_owned(),
                "qrcode-agent-cli help run --format json".to_owned(),
            ],
        }),
        [command] if command == "run" => Some(HelpDoc {
            command_path: vec!["qrcode-agent-cli".to_owned(), "run".to_owned()],
            summary: "Generate one QR code from inline content".to_owned(),
            description: "Text rendering writes the QR code itself to stdout. Image rendering writes a PNG file and returns structured metadata on stdout.".to_owned(),
            subcommands: Vec::new(),
            options: vec![
                option("content", "string", false, None, "Inline text content to encode as a QR code", Vec::new()),
                option("render", "enum", false, Some("text"), "Render mode to use for this invocation", enum_values(&["text", "image"])),
                option("output", "path", false, Some("./qrcode.png when render=image"), "Where to write the PNG image. Ignored in text mode.", Vec::new()),
                option("size", "int", false, Some(&DEFAULT_IMAGE_SIZE.to_string()), "Square PNG size in pixels when render=image", Vec::new()),
                option("format", "enum", false, Some("yaml"), "Structured success or error format for non-text responses", format_list()),
            ],
            output_formats: vec![
                "text".to_owned(),
                "yaml".to_owned(),
                "json".to_owned(),
                "toml".to_owned(),
            ],
            runtime_directories: runtime_paths.clone(),
            active_context,
            examples: vec![
                "qrcode-agent-cli run \"hello from terminal\" --render text".to_owned(),
                "qrcode-agent-cli run \"hello from png\" --render image --output ./hello.png --format json".to_owned(),
            ],
        }),
        [command] if command == "paths" => Some(HelpDoc {
            command_path: vec!["qrcode-agent-cli".to_owned(), "paths".to_owned()],
            summary: "Show config, data, state, cache, and context-file locations".to_owned(),
            description: "This command reports user-scoped runtime directories and any environment variable overrides that can redirect them for local testing.".to_owned(),
            subcommands: Vec::new(),
            options: vec![option(
                "format",
                "enum",
                false,
                Some("yaml"),
                "Structured output format",
                format_list(),
            )],
            output_formats: format_list(),
            runtime_directories: runtime_paths.clone(),
            active_context,
            examples: vec!["qrcode-agent-cli paths --format toml".to_owned()],
        }),
        [command] if command == "context" => Some(HelpDoc {
            command_path: vec!["qrcode-agent-cli".to_owned(), "context".to_owned()],
            summary: "Inspect or persist default QR rendering settings".to_owned(),
            description: "The active context stores default render settings in a user-scoped config file. `run` reads those defaults and lets invocation flags override them without mutating the saved file.".to_owned(),
            subcommands: vec!["show".to_owned(), "set".to_owned(), "clear".to_owned()],
            options: Vec::new(),
            output_formats: format_list(),
            runtime_directories: runtime_paths.clone(),
            active_context,
            examples: vec![
                "qrcode-agent-cli context show --format json".to_owned(),
                "qrcode-agent-cli context set --render image --size 512".to_owned(),
            ],
        }),
        [command, subcommand] if command == "context" && subcommand == "show" => Some(HelpDoc {
            command_path: vec![
                "qrcode-agent-cli".to_owned(),
                "context".to_owned(),
                "show".to_owned(),
            ],
            summary: "Show the persisted active context".to_owned(),
            description: "Returns the saved default render mode, default PNG output path, and default image size together with the runtime-directory layout.".to_owned(),
            subcommands: Vec::new(),
            options: vec![option(
                "format",
                "enum",
                false,
                Some("yaml"),
                "Structured output format",
                format_list(),
            )],
            output_formats: format_list(),
            runtime_directories: runtime_paths.clone(),
            active_context,
            examples: vec!["qrcode-agent-cli context show --format yaml".to_owned()],
        }),
        [command, subcommand] if command == "context" && subcommand == "set" => Some(HelpDoc {
            command_path: vec![
                "qrcode-agent-cli".to_owned(),
                "context".to_owned(),
                "set".to_owned(),
            ],
            summary: "Persist new default render settings".to_owned(),
            description: "Any provided fields are written to the active-context file. Later `run` invocations inherit these defaults unless the invocation passes explicit overrides.".to_owned(),
            subcommands: Vec::new(),
            options: vec![
                option("render", "enum", false, None, "Persist the default render mode", enum_values(&["text", "image"])),
                option("output", "path", false, None, "Persist the default PNG output path", Vec::new()),
                option("size", "int", false, None, "Persist the default PNG size in pixels", Vec::new()),
                option("format", "enum", false, Some("yaml"), "Structured output format", format_list()),
            ],
            output_formats: format_list(),
            runtime_directories: runtime_paths.clone(),
            active_context,
            examples: vec![
                "qrcode-agent-cli context set --render text".to_owned(),
                "qrcode-agent-cli context set --render image --output ./saved.png --size 384 --format json".to_owned(),
            ],
        }),
        [command, subcommand] if command == "context" && subcommand == "clear" => Some(HelpDoc {
            command_path: vec![
                "qrcode-agent-cli".to_owned(),
                "context".to_owned(),
                "clear".to_owned(),
            ],
            summary: "Delete the persisted active context".to_owned(),
            description: "After clearing the active context, `run` falls back to built-in defaults again.".to_owned(),
            subcommands: Vec::new(),
            options: vec![option(
                "format",
                "enum",
                false,
                Some("yaml"),
                "Structured output format",
                format_list(),
            )],
            output_formats: format_list(),
            runtime_directories: runtime_paths.clone(),
            active_context,
            examples: vec!["qrcode-agent-cli context clear --format json".to_owned()],
        }),
        [command] if command == "help" => Some(HelpDoc {
            command_path: vec!["qrcode-agent-cli".to_owned(), "help".to_owned()],
            summary: "Return structured help for the CLI or a specific command path".to_owned(),
            description: "This command is the machine-readable help surface. Plain-text help stays on `--help` for people and shells.".to_owned(),
            subcommands: Vec::new(),
            options: vec![
                option("command_path", "string[]", false, Some("[]"), "Optional command path such as `run` or `context show`", Vec::new()),
                option("format", "enum", false, Some("yaml"), "Structured output format", format_list()),
            ],
            output_formats: format_list(),
            runtime_directories: runtime_paths.clone(),
            active_context,
            examples: vec![
                "qrcode-agent-cli help --format yaml".to_owned(),
                "qrcode-agent-cli help context set --format json".to_owned(),
            ],
        }),
        _ => None,
    }
}

fn option(
    name: &str,
    value_type: &str,
    required: bool,
    default: Option<&str>,
    description: &str,
    enum_values: Vec<String>,
) -> HelpOption {
    HelpOption {
        name: name.to_owned(),
        value_type: value_type.to_owned(),
        required,
        default: default.map(str::to_owned),
        description: description.to_owned(),
        enum_values,
    }
}

fn format_list() -> Vec<String> {
    vec!["yaml".to_owned(), "json".to_owned(), "toml".to_owned()]
}

fn enum_values(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}
