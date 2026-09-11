use std::io::{self, IsTerminal, Read, Write};

use clap::Subcommand;
use serde::Serialize;

use super::{
    CliError, OutputFormat, Provider, print_product_json, provider_name, secure_credentials,
};

const MAX_STDIN_SECRET_BYTES: u64 = 16 * 1024;
const AUTH_PROVIDERS: [Provider; 4] = [
    Provider::Mistral,
    Provider::Google,
    Provider::Groq,
    Provider::Nvidia,
];

#[derive(Debug, Subcommand)]
pub(crate) enum AuthCommand {
    /// Save or replace one provider credential in the native OS credential store.
    Login {
        /// Provider to authenticate. Omit only in an interactive terminal to use the picker.
        #[arg(value_enum)]
        provider: Option<Provider>,
        /// Read the credential from stdin. Provider must be explicit; input is not echoed by Reason.
        #[arg(long, conflicts_with = "from_env")]
        stdin: bool,
        /// Copy the provider's supported environment variable into the OS credential store.
        #[arg(long, conflicts_with = "stdin")]
        from_env: bool,
        /// Explicitly replace an existing OS-store credential.
        #[arg(long)]
        replace: bool,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Show credential source/status without exposing credential values.
    Status {
        /// Optional provider. Omit to inspect all supported providers.
        #[arg(value_enum)]
        provider: Option<Provider>,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// List all supported providers and their credential source/status.
    List {
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Delete one provider credential from the native OS credential store.
    Logout {
        /// Provider to remove. Omit only in an interactive terminal to use the picker.
        #[arg(value_enum)]
        provider: Option<Provider>,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
}

impl AuthCommand {
    pub(crate) const fn format(&self) -> OutputFormat {
        match self {
            Self::Login { format, .. }
            | Self::Status { format, .. }
            | Self::List { format }
            | Self::Logout { format, .. } => *format,
        }
    }
}

#[derive(Debug, Serialize)]
struct AuthProviderStatus {
    provider: &'static str,
    account: &'static str,
    environment_variable: &'static str,
    environment: &'static str,
    os_store: &'static str,
    effective_source: &'static str,
}

#[derive(Debug, Serialize)]
struct AuthStatusOutput {
    providers: Vec<AuthProviderStatus>,
}

#[derive(Debug, Serialize)]
struct AuthLoginOutput {
    operation: &'static str,
    provider: &'static str,
    account: &'static str,
    stored: bool,
    replaced: bool,
    effective_source: &'static str,
}

#[derive(Debug, Serialize)]
struct AuthLogoutOutput {
    operation: &'static str,
    provider: &'static str,
    account: &'static str,
    removed: bool,
    effective_source: &'static str,
}

pub(crate) fn run(command: AuthCommand) -> Result<(), CliError> {
    match command {
        AuthCommand::Login {
            provider,
            stdin,
            from_env,
            replace,
            format,
        } => login(provider, stdin, from_env, replace, format),
        AuthCommand::Status { provider, format } => status(provider, format),
        AuthCommand::List { format } => status(None, format),
        AuthCommand::Logout { provider, format } => logout(provider, format),
    }
}

fn login(
    provider: Option<Provider>,
    from_stdin: bool,
    from_env: bool,
    replace: bool,
    format: OutputFormat,
) -> Result<(), CliError> {
    let provider = select_provider(provider, !from_stdin, "login")?;
    let provider = secure_credentials::canonical_provider(provider);
    let existed = secure_credentials::stored_provider_credential_exists(provider)
        .map_err(credential_error)?;
    if existed && !replace {
        return Err(CliError::new(
            "credential_exists",
            format!(
                "a stored {} credential already exists; use `reason auth login {} --replace` to rotate it",
                provider_name(provider),
                provider_name(provider)
            ),
        ));
    }

    let secret = if from_stdin {
        read_secret_from_stdin()?
    } else if from_env {
        secure_credentials::provider_environment_credential(provider)
            .map_err(credential_error)?
            .ok_or_else(|| {
                CliError::new(
                    "credentials",
                    format!(
                        "{} is not set; set it or use hidden interactive input / --stdin",
                        secure_credentials::credential_env(provider)
                    ),
                )
            })?
    } else {
        read_secret_interactively(provider)?
    };

    secure_credentials::save_provider_credential(provider, &secret).map_err(credential_error)?;
    let observed =
        secure_credentials::inspect_provider_credential(provider).map_err(credential_error)?;
    let identity = secure_credentials::credential_identity(provider);
    let output = AuthLoginOutput {
        operation: "login",
        provider: provider_name(provider),
        account: identity.account,
        stored: true,
        replaced: existed,
        effective_source: observed.effective.as_str(),
    };
    emit_login(&output, format)
}

fn status(provider: Option<Provider>, format: OutputFormat) -> Result<(), CliError> {
    let providers = match provider {
        Some(provider) => vec![secure_credentials::canonical_provider(provider)],
        None => AUTH_PROVIDERS.to_vec(),
    };
    let mut rows = Vec::with_capacity(providers.len());
    for provider in providers {
        rows.push(provider_status(provider)?);
    }
    let output = AuthStatusOutput { providers: rows };
    match format {
        OutputFormat::Json => print_product_json("auth", &output).map_err(CliError::from),
        OutputFormat::Human => {
            for row in &output.providers {
                println!(
                    "{}: effective={} environment={} os_store={} account={}",
                    row.provider, row.effective_source, row.environment, row.os_store, row.account
                );
            }
            Ok(())
        }
    }
}

fn logout(provider: Option<Provider>, format: OutputFormat) -> Result<(), CliError> {
    let provider = select_provider(provider, true, "logout")?;
    let provider = secure_credentials::canonical_provider(provider);
    let removed =
        secure_credentials::delete_provider_credential(provider).map_err(credential_error)?;
    let observed =
        secure_credentials::inspect_provider_credential(provider).map_err(credential_error)?;
    let identity = secure_credentials::credential_identity(provider);
    let output = AuthLogoutOutput {
        operation: "logout",
        provider: provider_name(provider),
        account: identity.account,
        removed,
        effective_source: observed.effective.as_str(),
    };
    match format {
        OutputFormat::Json => print_product_json("auth", &output).map_err(CliError::from),
        OutputFormat::Human => {
            if removed {
                println!(
                    "Removed stored {} credential for {}.",
                    output.provider, output.account
                );
            } else {
                println!(
                    "No stored {} credential existed for {}.",
                    output.provider, output.account
                );
            }
            if output.effective_source == "environment" {
                println!(
                    "{} is still set and remains the active credential source.",
                    secure_credentials::credential_env(provider)
                );
            }
            Ok(())
        }
    }
}

fn provider_status(provider: Provider) -> Result<AuthProviderStatus, CliError> {
    let provider = secure_credentials::canonical_provider(provider);
    let observed =
        secure_credentials::inspect_provider_credential(provider).map_err(credential_error)?;
    let identity = secure_credentials::credential_identity(provider);
    Ok(AuthProviderStatus {
        provider: provider_name(provider),
        account: identity.account,
        environment_variable: secure_credentials::credential_env(provider),
        environment: observed.environment.as_str(),
        os_store: observed.os_store.as_str(),
        effective_source: observed.effective.as_str(),
    })
}

fn emit_login(output: &AuthLoginOutput, format: OutputFormat) -> Result<(), CliError> {
    match format {
        OutputFormat::Json => print_product_json("auth", output).map_err(CliError::from),
        OutputFormat::Human => {
            let verb = if output.replaced {
                "Replaced"
            } else {
                "Stored"
            };
            println!(
                "{verb} {} credential in the native OS credential store ({}).",
                output.provider, output.account
            );
            if output.effective_source == "environment" {
                println!(
                    "The provider environment variable is currently set and overrides the stored credential."
                );
            }
            Ok(())
        }
    }
}

fn read_secret_interactively(provider: Provider) -> Result<String, CliError> {
    if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        return Err(CliError::new(
            "auth_input",
            "interactive credential entry requires a terminal; use --stdin or --from-env for automation",
        ));
    }
    let secret = rpassword::prompt_password(format!("{} API key: ", provider_name(provider)))
        .map_err(|error| {
            CliError::new(
                "auth_input",
                format!("could not read hidden credential input: {error}"),
            )
        })?;
    validate_secret(secret)
}

fn read_secret_from_stdin() -> Result<String, CliError> {
    let mut bytes = Vec::new();
    io::stdin()
        .take(MAX_STDIN_SECRET_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            CliError::new(
                "auth_input",
                format!("could not read credential from stdin: {error}"),
            )
        })?;
    parse_stdin_secret(bytes)
}

fn parse_stdin_secret(mut bytes: Vec<u8>) -> Result<String, CliError> {
    if bytes.len() as u64 > MAX_STDIN_SECRET_BYTES {
        return Err(CliError::new(
            "auth_input",
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
            "auth_input",
            "credential stdin must contain exactly one line",
        ));
    }
    let secret = String::from_utf8(bytes)
        .map_err(|_| CliError::new("auth_input", "credential stdin is not valid UTF-8"))?;
    validate_secret(secret)
}

fn validate_secret(secret: String) -> Result<String, CliError> {
    if secret.trim().is_empty() {
        return Err(CliError::new(
            "credentials",
            "provider credential must not be empty",
        ));
    }
    Ok(secret)
}

fn select_provider(
    provider: Option<Provider>,
    allow_picker: bool,
    operation: &str,
) -> Result<Provider, CliError> {
    if let Some(provider) = provider {
        return Ok(secure_credentials::canonical_provider(provider));
    }
    if !allow_picker {
        return Err(CliError::new(
            "auth_input",
            format!("provider is required for `reason auth {operation}` with --stdin"),
        ));
    }
    if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        return Err(CliError::new(
            "auth_input",
            format!(
                "provider is required for non-interactive `reason auth {operation}`; choose mistral, google, groq, or nvidia"
            ),
        ));
    }

    eprintln!("Select provider:");
    eprintln!("  1) mistral");
    eprintln!("  2) google");
    eprintln!("  3) groq");
    eprintln!("  4) nvidia");
    eprint!("Provider [1-4]: ");
    io::stderr().flush().map_err(|error| {
        CliError::new(
            "auth_input",
            format!("could not write provider prompt: {error}"),
        )
    })?;
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).map_err(|error| {
        CliError::new(
            "auth_input",
            format!("could not read provider selection: {error}"),
        )
    })?;
    parse_provider_choice(&choice).ok_or_else(|| {
        CliError::new(
            "auth_input",
            "invalid provider selection; choose 1-4 or mistral/google/groq/nvidia",
        )
    })
}

fn parse_provider_choice(choice: &str) -> Option<Provider> {
    match choice.trim().to_ascii_lowercase().as_str() {
        "1" | "mistral" => Some(Provider::Mistral),
        "2" | "google" | "gemini" | "gemma" => Some(Provider::Google),
        "3" | "groq" => Some(Provider::Groq),
        "4" | "nvidia" => Some(Provider::Nvidia),
        _ => None,
    }
}

fn credential_error(error: secure_credentials::CredentialError) -> CliError {
    CliError::new(error.failure_class(), error.message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdin_secret_accepts_one_line_and_strips_only_line_ending() {
        assert_eq!(parse_stdin_secret(b"abc123\n".to_vec()).unwrap(), "abc123");
        assert_eq!(
            parse_stdin_secret(b"abc123\r\n".to_vec()).unwrap(),
            "abc123"
        );
        assert_eq!(
            parse_stdin_secret(b" abc123 ".to_vec()).unwrap(),
            " abc123 "
        );
    }

    #[test]
    fn stdin_secret_rejects_multiline_empty_invalid_utf8_and_oversize() {
        assert!(parse_stdin_secret(b"a\nb\n".to_vec()).is_err());
        assert!(parse_stdin_secret(b" \n".to_vec()).is_err());
        assert!(parse_stdin_secret(vec![0xff]).is_err());
        assert!(parse_stdin_secret(vec![b'x'; MAX_STDIN_SECRET_BYTES as usize + 1]).is_err());
    }

    #[test]
    fn provider_picker_accepts_numbers_names_and_google_aliases() {
        assert_eq!(parse_provider_choice("1"), Some(Provider::Mistral));
        assert_eq!(parse_provider_choice("google"), Some(Provider::Google));
        assert_eq!(parse_provider_choice("gemini"), Some(Provider::Google));
        assert_eq!(parse_provider_choice("gemma"), Some(Provider::Google));
        assert_eq!(parse_provider_choice("3"), Some(Provider::Groq));
        assert_eq!(parse_provider_choice("NVIDIA"), Some(Provider::Nvidia));
        assert_eq!(parse_provider_choice("wat"), None);
    }
}
