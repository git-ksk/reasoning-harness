use std::{env, fs, path::Path};

use clap::Args;
use reasoning_harness_core::{ModelOutputFormat, ModelRequest};
use serde::Serialize;

use super::{
    CliError, LiveGenerator, OutputFormat, Provider, lifecycle, load_cli_config, managed_session,
    mcp_commands, model_catalog, model_error_class, print_product_json, project_config_path,
    project_trust, provider_name, secure_credentials, user_config_path,
};

const DOCTOR_SURFACE_ID: &str = "reason-doctor-v1";
const PROVIDERS: [Provider; 4] = [
    Provider::Mistral,
    Provider::Google,
    Provider::Groq,
    Provider::Nvidia,
];

#[derive(Debug, Args)]
pub(crate) struct DoctorArgs {
    /// Run bounded provider/MCP/update network checks. Provider checks may consume quota or incur cost.
    #[arg(long)]
    live_check: bool,
    #[arg(long, value_enum, default_value_t)]
    format: OutputFormat,
}

impl DoctorArgs {
    pub(crate) const fn format(&self) -> OutputFormat {
        self.format
    }
}

#[derive(Debug, Serialize)]
struct VersionDiagnostic {
    cli: &'static str,
    engine: &'static str,
}

#[derive(Debug, Serialize)]
struct InstallationDiagnostic {
    executable_path: String,
    method: &'static str,
}

#[derive(Debug, Serialize)]
struct ConfigDiagnostic {
    status: &'static str,
    user_path: Option<String>,
    user_present: bool,
    user_path_status: &'static str,
    project_path: Option<String>,
    project_present: bool,
    project_path_status: &'static str,
    effective_sources: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct CredentialDiagnostic {
    provider: &'static str,
    environment: &'static str,
    os_store: &'static str,
    effective_source: &'static str,
}

#[derive(Debug, Serialize)]
struct SecretStoreDiagnostic {
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct ProviderDiagnostic {
    status: &'static str,
    provider: Option<&'static str>,
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compatibility: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model_availability: Option<&'static str>,
    local_readiness: &'static str,
    live_readiness: &'static str,
    live_provider_attempts: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    live_total_tokens: Option<u64>,
}

#[derive(Debug, Serialize)]
struct PathDiagnostic {
    path: Option<String>,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct TrustDiagnostic {
    status: &'static str,
    project_path: Option<String>,
    config_present: bool,
    high_risk_config: bool,
    trusted: bool,
}

#[derive(Debug, Serialize)]
struct UpdateDiagnostic {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    available_cli_version: Option<String>,
}

#[derive(Debug, Serialize)]
struct DoctorIssue {
    severity: &'static str,
    component: &'static str,
    failure_class: &'static str,
    message: String,
    recovery: String,
}

#[derive(Debug, Serialize)]
struct DoctorOutput {
    doctor_surface: &'static str,
    status: &'static str,
    live_check: bool,
    versions: VersionDiagnostic,
    installation: InstallationDiagnostic,
    config: ConfigDiagnostic,
    credentials: Vec<CredentialDiagnostic>,
    secret_store: SecretStoreDiagnostic,
    provider: ProviderDiagnostic,
    sessions: PathDiagnostic,
    project_trust: TrustDiagnostic,
    mcp: mcp_commands::McpDoctorStatus,
    update: UpdateDiagnostic,
    issues: Vec<DoctorIssue>,
}

pub(crate) async fn run(args: DoctorArgs) -> Result<(), CliError> {
    let mut issues = Vec::new();

    let executable = env::current_exe().map_err(|error| {
        CliError::new(
            "doctor_installation",
            format!("cannot determine current Reason executable: {error}"),
        )
    })?;
    let installation = InstallationDiagnostic {
        executable_path: executable.display().to_string(),
        method: installation_method(&executable),
    };

    let user_path = user_config_path();
    let project_path = project_config_path();
    let user_present = user_path.as_ref().is_some_and(|path| path.is_file());
    let project_present = project_path.as_ref().is_some_and(|path| path.is_file());

    let loaded = match load_cli_config(None) {
        Ok(loaded) => Some(loaded),
        Err(error) => {
            issues.push(DoctorIssue {
                severity: "error",
                component: "config",
                failure_class: "configuration",
                message: error,
                recovery: "reason config sources".into(),
            });
            None
        }
    };
    let config = ConfigDiagnostic {
        status: if loaded.is_some() { "valid" } else { "invalid" },
        user_path: user_path.as_ref().map(|path| path.display().to_string()),
        user_present,
        user_path_status: user_path
            .as_ref()
            .map_or("unavailable", |path| file_path_status(path)),
        project_path: project_path.as_ref().map(|path| path.display().to_string()),
        project_present,
        project_path_status: project_path
            .as_ref()
            .map_or("unavailable", |path| file_path_status(path)),
        effective_sources: loaded
            .as_ref()
            .map(|loaded| loaded.sources.clone())
            .unwrap_or_default(),
    };

    let mut credentials = Vec::new();
    let mut store_observed_available = false;
    let mut store_observed_unavailable = false;
    for provider in PROVIDERS {
        match secure_credentials::inspect_provider_credential(provider) {
            Ok(status) => {
                if status.os_store == secure_credentials::StoredCredentialState::Unavailable {
                    store_observed_unavailable = true;
                } else {
                    store_observed_available = true;
                }
                credentials.push(CredentialDiagnostic {
                    provider: provider_name(provider),
                    environment: status.environment.as_str(),
                    os_store: status.os_store.as_str(),
                    effective_source: status.effective.as_str(),
                });
            }
            Err(error) => {
                issues.push(DoctorIssue {
                    severity: "error",
                    component: "credential_store",
                    failure_class: error.failure_class(),
                    message: error.message,
                    recovery: "reason auth status".into(),
                });
                credentials.push(CredentialDiagnostic {
                    provider: provider_name(provider),
                    environment: "unknown",
                    os_store: "error",
                    effective_source: "unknown",
                });
            }
        }
    }
    let secret_store = SecretStoreDiagnostic {
        status: if store_observed_available {
            "available"
        } else if store_observed_unavailable {
            "unavailable"
        } else {
            "error"
        },
    };

    let provider = diagnose_provider(loaded.as_ref(), args.live_check, &mut issues).await;

    let sessions = match managed_session::managed_root_path() {
        Ok(path) => PathDiagnostic {
            status: path_status(&path),
            path: Some(path.display().to_string()),
        },
        Err(error) => {
            issues.push(DoctorIssue {
                severity: "error",
                component: "sessions",
                failure_class: error.failure_class,
                message: error.message,
                recovery: "reason session list".into(),
            });
            PathDiagnostic {
                path: None,
                status: "unavailable",
            }
        }
    };

    let project_trust = match project_trust::status(None) {
        Ok(status) => {
            if status.high_risk_config && !status.trusted {
                issues.push(DoctorIssue {
                    severity: "warning",
                    component: "project_trust",
                    failure_class: "project_trust_required",
                    message: "project configuration contains executable/acquisition settings that are not trusted".into(),
                    recovery: "reason trust status".into(),
                });
            }
            TrustDiagnostic {
                status: status.state,
                project_path: Some(status.project_path),
                config_present: status.config_exists,
                high_risk_config: status.high_risk_config,
                trusted: status.trusted,
            }
        }
        Err(error) => {
            issues.push(DoctorIssue {
                severity: "warning",
                component: "project_trust",
                failure_class: "project_trust",
                message: error,
                recovery: "reason trust status".into(),
            });
            TrustDiagnostic {
                status: "unavailable",
                project_path: None,
                config_present: false,
                high_risk_config: false,
                trusted: false,
            }
        }
    };

    let mcp = match mcp_commands::doctor_status(args.live_check) {
        Ok(status) => status,
        Err(error) => {
            issues.push(DoctorIssue {
                severity: "warning",
                component: "mcp",
                failure_class: error.failure_class,
                message: error.message,
                recovery: "reason mcp list".into(),
            });
            mcp_commands::McpDoctorStatus {
                configured: true,
                name: None,
                transport: None,
                readiness: "failed",
                selected_tool: None,
                negotiated_protocol_version: None,
            }
        }
    };

    let update = if args.live_check {
        match lifecycle::diagnostic_update_availability().await {
            Ok(Some(version)) => UpdateDiagnostic {
                status: "available",
                available_cli_version: Some(version),
            },
            Ok(None) => UpdateDiagnostic {
                status: "current",
                available_cli_version: None,
            },
            Err(error) => {
                issues.push(DoctorIssue {
                    severity: "warning",
                    component: "update",
                    failure_class: error.failure_class,
                    message: error.message,
                    recovery: "reason update --check".into(),
                });
                UpdateDiagnostic {
                    status: "unavailable",
                    available_cli_version: None,
                }
            }
        }
    } else {
        UpdateDiagnostic {
            status: "skipped",
            available_cli_version: None,
        }
    };

    let output = DoctorOutput {
        doctor_surface: DOCTOR_SURFACE_ID,
        status: if issues.iter().any(|issue| issue.severity == "error") {
            "attention"
        } else if issues.is_empty() {
            "ok"
        } else {
            "attention"
        },
        live_check: args.live_check,
        versions: VersionDiagnostic {
            cli: env!("CARGO_PKG_VERSION"),
            engine: reasoning_harness_core::ENGINE_VERSION,
        },
        installation,
        config,
        credentials,
        secret_store,
        provider,
        sessions,
        project_trust,
        mcp,
        update,
        issues,
    };

    emit(&output, args.format)
}

async fn diagnose_provider(
    loaded: Option<&super::LoadedCliConfig>,
    live_check: bool,
    issues: &mut Vec<DoctorIssue>,
) -> ProviderDiagnostic {
    let Some(loaded) = loaded else {
        return ProviderDiagnostic {
            status: "unavailable",
            provider: None,
            model: None,
            compatibility: None,
            model_availability: None,
            local_readiness: "blocked",
            live_readiness: "not_run",
            live_provider_attempts: 0,
            live_total_tokens: None,
        };
    };
    let provider = loaded.config.run.provider;
    let model = loaded.config.run.model.clone();
    let (Some(provider), Some(model)) = (provider, model.clone()) else {
        issues.push(DoctorIssue {
            severity: "warning",
            component: "provider",
            failure_class: "setup_required",
            message: "default provider/model is not fully configured".into(),
            recovery: "reason setup".into(),
        });
        return ProviderDiagnostic {
            status: "not_configured",
            provider: provider.map(provider_name),
            model,
            compatibility: None,
            model_availability: None,
            local_readiness: "not_configured",
            live_readiness: "not_run",
            live_provider_attempts: 0,
            live_total_tokens: None,
        };
    };

    let model_availability = model_catalog::model_availability(provider, &model);
    let compatibility = match model_catalog::validate_general_use_model(provider, &model) {
        Ok(compatibility) => Some(compatibility),
        Err(error) => {
            issues.push(DoctorIssue {
                severity: "error",
                component: "provider",
                failure_class: error.failure_class,
                message: error.message,
                recovery: format!("reason models {}", provider_name(provider)),
            });
            None
        }
    };
    let credential = secure_credentials::inspect_provider_credential(provider);
    let credential_ready = match credential {
        Ok(status) => matches!(
            status.effective,
            secure_credentials::EffectiveCredentialSource::Environment
                | secure_credentials::EffectiveCredentialSource::OsStore
        ),
        Err(_) => false,
    };
    if !credential_ready {
        issues.push(DoctorIssue {
            severity: "error",
            component: "provider",
            failure_class: "credentials",
            message: format!(
                "{} has no usable effective credential",
                provider_name(provider)
            ),
            recovery: format!("reason auth status {}", provider_name(provider)),
        });
    }
    let local_ready = compatibility.is_some() && credential_ready;
    if !live_check || !local_ready {
        return ProviderDiagnostic {
            status: if local_ready { "ready" } else { "blocked" },
            provider: Some(provider_name(provider)),
            model: Some(model),
            compatibility,
            model_availability: Some(model_availability),
            local_readiness: if local_ready { "ready" } else { "blocked" },
            live_readiness: if live_check { "blocked" } else { "skipped" },
            live_provider_attempts: 0,
            live_total_tokens: None,
        };
    }

    let generator = match LiveGenerator::try_from_provider(provider, &model) {
        Ok(generator) => generator,
        Err(error) => {
            issues.push(DoctorIssue {
                severity: "error",
                component: "provider_live",
                failure_class: error.failure_class,
                message: error.message,
                recovery: format!("reason auth status {}", provider_name(provider)),
            });
            return ProviderDiagnostic {
                status: "blocked",
                provider: Some(provider_name(provider)),
                model: Some(model),
                compatibility,
                model_availability: Some(model_availability),
                local_readiness: "ready",
                live_readiness: "failed",
                live_provider_attempts: 0,
                live_total_tokens: None,
            };
        }
    };
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
        Ok(response) => ProviderDiagnostic {
            status: "ready",
            provider: Some(provider_name(provider)),
            model: Some(model),
            compatibility,
            model_availability: Some(model_availability),
            local_readiness: "ready",
            live_readiness: "passed",
            live_provider_attempts: response.provider_attempts,
            live_total_tokens: response.usage.total_tokens,
        },
        Err(error) => {
            issues.push(DoctorIssue {
                severity: "error",
                component: "provider_live",
                failure_class: model_error_class(error.kind),
                message: error.message,
                recovery: "reason doctor --live-check".into(),
            });
            ProviderDiagnostic {
                status: "blocked",
                provider: Some(provider_name(provider)),
                model: Some(model),
                compatibility,
                model_availability: Some(model_availability),
                local_readiness: "ready",
                live_readiness: "failed",
                live_provider_attempts: 0,
                live_total_tokens: None,
            }
        }
    }
}

fn path_status(path: &Path) -> &'static str {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => "ready",
        Ok(_) => "invalid_type",
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "not_created",
        Err(_) => "unavailable",
    }
}

fn file_path_status(path: &Path) -> &'static str {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => "ready",
        Ok(_) => "invalid_type",
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "not_created",
        Err(_) => "unavailable",
    }
}

fn installation_method(path: &Path) -> &'static str {
    let value = path.to_string_lossy().replace('\\', "/");
    if value.contains("/target/debug/") || value.contains("/target/release/") {
        "development_build"
    } else if value.contains("/.cargo/bin/") {
        "cargo"
    } else if value.contains("/opt/homebrew/")
        || value.contains("/usr/local/Cellar/")
        || value.contains("/usr/local/opt/")
    {
        "homebrew"
    } else {
        "unknown"
    }
}

fn emit(output: &DoctorOutput, format: OutputFormat) -> Result<(), CliError> {
    match format {
        OutputFormat::Json => print_product_json("doctor", output).map_err(CliError::from),
        OutputFormat::Human => {
            println!("Reason doctor: {}", output.status);
            println!("CLI version: {}", output.versions.cli);
            println!("Harness Engine version: {}", output.versions.engine);
            println!(
                "Installation: {} ({})",
                output.installation.executable_path, output.installation.method
            );
            println!(
                "Config: {} sources={}",
                output.config.status,
                if output.config.effective_sources.is_empty() {
                    "default".into()
                } else {
                    output.config.effective_sources.join(",")
                }
            );
            println!("Credential store: {}", output.secret_store.status);
            for credential in &output.credentials {
                println!(
                    "Credential {}: environment={} os_store={} effective={}",
                    credential.provider,
                    credential.environment,
                    credential.os_store,
                    credential.effective_source
                );
            }
            println!(
                "Provider: {} model={} availability={} local={} live={}",
                output.provider.provider.unwrap_or("-"),
                output.provider.model.as_deref().unwrap_or("-"),
                output.provider.model_availability.unwrap_or("-"),
                output.provider.local_readiness,
                output.provider.live_readiness
            );
            println!(
                "Sessions: {} {}",
                output.sessions.status,
                output.sessions.path.as_deref().unwrap_or("-")
            );
            println!(
                "Project trust: {} high_risk={} trusted={}",
                output.project_trust.status,
                output.project_trust.high_risk_config,
                output.project_trust.trusted
            );
            println!(
                "MCP: configured={} readiness={} name={}",
                output.mcp.configured,
                output.mcp.readiness,
                output.mcp.name.as_deref().unwrap_or("-")
            );
            println!("Update: {}", output.update.status);
            if !output.live_check {
                println!(
                    "Live checks: skipped (use `reason doctor --live-check`; provider checks may consume quota or incur cost)"
                );
            }
            for issue in &output.issues {
                println!(
                    "[{}] {}: {} | next: {}",
                    issue.severity, issue.component, issue.message, issue.recovery
                );
            }
            Ok(())
        }
    }
}
