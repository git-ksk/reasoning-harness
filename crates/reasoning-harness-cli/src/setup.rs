use std::io::{self, IsTerminal, Read, Write};

use clap::Args;
use reasoning_harness_core::{ModelOutputFormat, ModelRequest};
use serde::Serialize;

use super::{
    CliError, LiveGenerator, OutputFormat, Provider, model_catalog, model_error_class,
    print_product_json, provider_name, secure_credentials,
};

const MAX_STDIN_SECRET_BYTES: u64 = 16 * 1024;
const FIRST_COMMAND: &str = "reason \"Report the verified service status\" --fact service.status=healthy --hypothesis service.status=healthy";

#[derive(Debug, Args)]
pub(crate) struct SetupArgs {
    #[arg(long, value_enum)]
    provider: Option<Provider>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long)]
    non_interactive: bool,
    #[arg(long, conflicts_with = "from_env")]
    credential_stdin: bool,
    #[arg(long, conflicts_with = "credential_stdin")]
    from_env: bool,
    #[arg(long)]
    replace_credential: bool,
    #[arg(long, conflicts_with = "skip_live_check")]
    live_check: bool,
    #[arg(long, conflicts_with = "live_check")]
    skip_live_check: bool,
    #[arg(long, value_enum, default_value_t)]
    format: OutputFormat,
}

impl SetupArgs {
    pub(crate) const fn format(&self) -> OutputFormat {
        self.format
    }
}

#[derive(Debug, Serialize)]
struct SetupOutput {
    provider: &'static str,
    model: String,
    compatibility: &'static str,
    credential_source: &'static str,
    config_path: String,
    local_readiness: &'static str,
    live_readiness: &'static str,
    live_provider_attempts: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    live_total_tokens: Option<u64>,
    first_party_telemetry: bool,
    outbound_data_notice: &'static str,
    credential_storage_notice: &'static str,
    first_command: &'static str,
}

pub(crate) async fn run(args: SetupArgs) -> Result<(), CliError> {
    if args.non_interactive && args.provider.is_none() {
        return Err(CliError::new(
            "setup_input",
            "--provider is required with --non-interactive",
        ));
    }
    if args.credential_stdin && !args.non_interactive {
        return Err(CliError::new(
            "setup_input",
            "--credential-stdin requires --non-interactive so stdin is not shared with prompts",
        ));
    }

    let provider = secure_credentials::canonical_provider(select_provider(
        args.provider,
        args.non_interactive,
    )?);
    let model = select_model(provider, args.model, args.non_interactive)?;
    let compatibility = model_catalog::validate_general_use_model(provider, &model)?;

    configure_credential(
        provider,
        args.credential_stdin,
        args.from_env,
        args.replace_credential,
        args.non_interactive,
    )?;

    let generator = LiveGenerator::try_from_provider(provider, &model)
        .map_err(|error| CliError::new(error.failure_class, error.message))?;
    let config_path = model_catalog::persist_user_default(provider, &model)?;
    let credential_source = secure_credentials::resolve_provider_credential(provider)
        .map_err(credential_error)?
        .source
        .as_str();

    let should_live_check = if args.live_check {
        true
    } else if args.skip_live_check || args.non_interactive {
        false
    } else {
        prompt_yes_no(
            "Run a minimal live provider readiness check now? It may consume quota or incur cost. [y/N]: ",
            false,
        )?
    };

    let (live_readiness, live_provider_attempts, live_total_tokens) = if should_live_check {
        let request = ModelRequest {
            task: "Respond with exactly READY.".into(),
            system: Some(
                "This is an operational connectivity check. Do not provide explanations.".into(),
            ),
            output_format: ModelOutputFormat::Text,
            max_tokens: Some(8),
            random_seed: Some(0),
            reasoning_preference: None,
        };
        match generator.adapter().generate(request).await {
            Ok(response) => (
                "passed",
                response.provider_attempts,
                response.usage.total_tokens,
            ),
            Err(error) => {
                return Err(CliError::new(
                    model_error_class(error.kind),
                    format!("live readiness check failed: {}", error.message),
                ));
            }
        }
    } else {
        ("skipped", 0, None)
    };

    let output = SetupOutput {
        provider: provider_name(provider),
        model,
        compatibility,
        credential_source,
        config_path,
        local_readiness: "passed",
        live_readiness,
        live_provider_attempts,
        live_total_tokens,
        first_party_telemetry: false,
        outbound_data_notice: "Tasks and untrusted context used for model generation are sent to the selected provider. Configured MCP/external resolvers receive only their bounded acquisition requests/arguments. Reason does not send first-party telemetry by default.",
        credential_storage_notice: "Provider credentials stay in the native OS credential store (or the explicit environment source) and are not written to Reason sessions, history, config, diagnostics, stdout, or stderr.",
        first_command: FIRST_COMMAND,
    };
    emit(&output, args.format)
}

fn select_provider(
    provider: Option<Provider>,
    non_interactive: bool,
) -> Result<Provider, CliError> {
    if let Some(provider) = provider {
        return Ok(provider);
    }
    if non_interactive || !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        return Err(CliError::new(
            "setup_input",
            "provider is required outside an interactive terminal; use --provider",
        ));
    }
    eprintln!("Choose a provider:");
    eprintln!("  1) mistral");
    eprintln!("  2) google");
    eprintln!("  3) groq");
    eprintln!("  4) nvidia (no current general-use default)");
    eprint!("Provider [1-4]: ");
    io::stderr().flush().map_err(input_error)?;
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).map_err(input_error)?;
    match choice.trim().to_ascii_lowercase().as_str() {
        "1" | "mistral" => Ok(Provider::Mistral),
        "2" | "google" | "gemini" | "gemma" => Ok(Provider::Google),
        "3" | "groq" => Ok(Provider::Groq),
        "4" | "nvidia" => Ok(Provider::Nvidia),
        _ => Err(CliError::new(
            "setup_input",
            "invalid provider selection; choose 1-4 or mistral/google/groq/nvidia",
        )),
    }
}

fn select_model(
    provider: Provider,
    model: Option<String>,
    non_interactive: bool,
) -> Result<String, CliError> {
    if let Some(model) = model {
        model_catalog::validate_general_use_model(provider, &model)?;
        return Ok(model);
    }
    let recommended = model_catalog::recommended_model(provider).ok_or_else(|| {
        CliError::new(
            "model_not_general_use",
            format!(
                "{} has no current curated general-use default; choose another provider or use explicit research --model outside setup",
                provider_name(provider)
            ),
        )
    })?;
    if non_interactive || !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        return Ok(recommended.to_string());
    }
    let models = model_catalog::general_use_models(provider);
    eprintln!("Choose a model:");
    for (index, (model, is_recommended, compatibility)) in models.iter().enumerate() {
        let marker = if *is_recommended { " recommended" } else { "" };
        eprintln!("  {}) {} [{}]{}", index + 1, model, compatibility, marker);
    }
    eprint!("Model [1]: ");
    io::stderr().flush().map_err(input_error)?;
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).map_err(input_error)?;
    let trimmed = choice.trim();
    if trimmed.is_empty() {
        return Ok(recommended.to_string());
    }
    if let Ok(index) = trimmed.parse::<usize>() {
        if let Some((model, _, _)) = models.get(index.saturating_sub(1)) {
            return Ok((*model).to_string());
        }
    }
    if models.iter().any(|(model, _, _)| *model == trimmed) {
        return Ok(trimmed.to_string());
    }
    Err(CliError::new(
        "setup_input",
        "invalid model selection; choose one of the listed general-use models",
    ))
}

fn configure_credential(
    provider: Provider,
    credential_stdin: bool,
    from_env: bool,
    replace: bool,
    non_interactive: bool,
) -> Result<(), CliError> {
    if credential_stdin || from_env {
        let exists = secure_credentials::stored_provider_credential_exists(provider)
            .map_err(credential_error)?;
        if exists && !replace {
            return Err(CliError::new(
                "credential_exists",
                "a native-store credential already exists; use --replace-credential to rotate it",
            ));
        }
        let secret = if credential_stdin {
            read_stdin_secret()?
        } else {
            secure_credentials::provider_environment_credential(provider)
                .map_err(credential_error)?
                .ok_or_else(|| {
                    CliError::new(
                        "credentials",
                        format!(
                            "{} is not set",
                            secure_credentials::credential_env(provider)
                        ),
                    )
                })?
        };
        secure_credentials::save_provider_credential(provider, &secret)
            .map_err(credential_error)?;
        return Ok(());
    }

    match secure_credentials::resolve_provider_credential(provider) {
        Ok(_) => Ok(()),
        Err(error) if error.failure_class() == "credentials" && !non_interactive => {
            if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
                return Err(credential_error(error));
            }
            let secret =
                rpassword::prompt_password(format!("{} API key: ", provider_name(provider)))
                    .map_err(input_error)?;
            if secret.trim().is_empty() {
                return Err(CliError::new(
                    "credentials",
                    "provider credential must not be empty",
                ));
            }
            secure_credentials::save_provider_credential(provider, &secret)
                .map_err(credential_error)
        }
        Err(error) => Err(credential_error(error)),
    }
}

fn read_stdin_secret() -> Result<String, CliError> {
    let mut bytes = Vec::new();
    io::stdin()
        .take(MAX_STDIN_SECRET_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(input_error)?;
    if bytes.len() as u64 > MAX_STDIN_SECRET_BYTES {
        return Err(CliError::new(
            "setup_input",
            format!("credential stdin exceeds the {MAX_STDIN_SECRET_BYTES} byte limit"),
        ));
    }
    if bytes.ends_with(b"\n") {
        bytes.pop();
        if bytes.ends_with(b"\r") {
            bytes.pop();
        }
    }
    if bytes.contains(&b'\n') || bytes.contains(&b'\r') {
        return Err(CliError::new(
            "setup_input",
            "credential stdin must contain exactly one line",
        ));
    }
    let secret = String::from_utf8(bytes)
        .map_err(|_| CliError::new("setup_input", "credential stdin is not valid UTF-8"))?;
    if secret.trim().is_empty() {
        return Err(CliError::new(
            "credentials",
            "provider credential must not be empty",
        ));
    }
    Ok(secret)
}

fn prompt_yes_no(prompt: &str, default: bool) -> Result<bool, CliError> {
    if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        return Ok(default);
    }
    eprint!("{prompt}");
    io::stderr().flush().map_err(input_error)?;
    let mut value = String::new();
    io::stdin().read_line(&mut value).map_err(input_error)?;
    match value.trim().to_ascii_lowercase().as_str() {
        "" => Ok(default),
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => Err(CliError::new("setup_input", "expected yes or no")),
    }
}

fn emit(output: &SetupOutput, format: OutputFormat) -> Result<(), CliError> {
    match format {
        OutputFormat::Json => print_product_json("setup", output).map_err(CliError::from),
        OutputFormat::Human => {
            println!("Setup complete.");
            println!("Provider: {}", output.provider);
            println!("Model: {} ({})", output.model, output.compatibility);
            println!("Credential source: {}", output.credential_source);
            println!("Config: {}", output.config_path);
            println!("Local readiness: {}", output.local_readiness);
            println!("Live readiness: {}", output.live_readiness);
            println!("Privacy: {}", output.outbound_data_notice);
            println!("Credentials: {}", output.credential_storage_notice);
            println!("Try this next:");
            println!("  {}", output.first_command);
            Ok(())
        }
    }
}

fn credential_error(error: secure_credentials::CredentialError) -> CliError {
    CliError::new(error.failure_class(), error.message)
}

fn input_error(error: io::Error) -> CliError {
    CliError::new("setup_input", format!("setup input/output failed: {error}"))
}
