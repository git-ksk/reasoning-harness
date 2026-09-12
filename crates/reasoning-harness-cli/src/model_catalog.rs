use std::{fs, io::Write, path::Path};

use clap::Subcommand;
use serde::Serialize;

use super::{
    CLI_CONFIG_CONTRACT_ID, CliError, CliFileConfig, OutputFormat, Provider, local_privacy,
    print_product_json, provider_name, secure_credentials, user_config_path,
};

const MODEL_CATALOG_VERSION: &str = "reason-model-catalog-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Compatibility {
    Validated,
    Observed,
    Limited,
}

impl Compatibility {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Validated => "validated",
            Self::Observed => "observed",
            Self::Limited => "limited",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ModelSpec {
    provider: Provider,
    model: &'static str,
    compatibility: Compatibility,
    general_use: bool,
    recommended: bool,
    evidence: &'static str,
    note: &'static str,
}

const MODELS: &[ModelSpec] = &[
    ModelSpec {
        provider: Provider::Mistral,
        model: "ministral-8b-latest",
        compatibility: Compatibility::Validated,
        general_use: true,
        recommended: true,
        evidence: "v0.4.2_release_acceptance",
        note: "Canonical release-acceptance model for the Mistral adapter.",
    },
    ModelSpec {
        provider: Provider::Mistral,
        model: "ministral-14b-latest",
        compatibility: Compatibility::Observed,
        general_use: true,
        recommended: false,
        evidence: "product_external_info_v4",
        note: "Completed the frozen v4 product workload; not the canonical release-acceptance identity.",
    },
    ModelSpec {
        provider: Provider::Google,
        model: "gemini-3.5-flash-lite",
        compatibility: Compatibility::Validated,
        general_use: true,
        recommended: true,
        evidence: "v0.4.2_release_acceptance",
        note: "Canonical Google Gemini release-acceptance model.",
    },
    ModelSpec {
        provider: Provider::Google,
        model: "gemma-4-31b-it",
        compatibility: Compatibility::Validated,
        general_use: true,
        recommended: false,
        evidence: "v0.4.2_release_acceptance",
        note: "Google-hosted Gemma identity with canonical release-acceptance evidence.",
    },
    ModelSpec {
        provider: Provider::Groq,
        model: "openai/gpt-oss-120b",
        compatibility: Compatibility::Validated,
        general_use: true,
        recommended: true,
        evidence: "v0.4.2_release_acceptance",
        note: "Canonical Groq release-acceptance model.",
    },
    ModelSpec {
        provider: Provider::Groq,
        model: "qwen/qwen3.8-27b",
        compatibility: Compatibility::Observed,
        general_use: true,
        recommended: false,
        evidence: "product_external_info_v4",
        note: "Completed the frozen v4 product workload with the current Groq adapter contract.",
    },
    ModelSpec {
        provider: Provider::Groq,
        model: "openai/gpt-oss-20b",
        compatibility: Compatibility::Observed,
        general_use: true,
        recommended: false,
        evidence: "product_external_info_v4",
        note: "Completed the frozen v4 product workload after bounded provider JSON-validation retry handling.",
    },
    ModelSpec {
        provider: Provider::Nvidia,
        model: "nvidia/nemotron-3.5-lightning-30b-a3b",
        compatibility: Compatibility::Limited,
        general_use: false,
        recommended: false,
        evidence: "semantic_d3_negative_control",
        note: "Research observations found protocol incompatibility in current semantic roles; explicit --model research use remains available.",
    },
];

#[derive(Debug, Subcommand)]
pub(crate) enum ModelCommand {
    /// Persist one curated provider/model pair as the user default.
    Set {
        #[arg(value_enum)]
        provider: Provider,
        model: String,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
}

impl ModelCommand {
    pub(crate) const fn format(&self) -> OutputFormat {
        match self {
            Self::Set { format, .. } => *format,
        }
    }
}

#[derive(Debug, Serialize)]
struct ModelRow {
    provider: &'static str,
    model: &'static str,
    compatibility: &'static str,
    general_use: bool,
    recommended: bool,
    evidence: &'static str,
    note: &'static str,
    configured_default: bool,
    credential_status: &'static str,
}

#[derive(Debug, Serialize)]
struct ConfiguredDefault {
    provider: &'static str,
    model: String,
    catalog_status: &'static str,
    credential_status: &'static str,
}

#[derive(Debug, Serialize)]
struct ModelsOutput {
    catalog_version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    configured_default: Option<ConfiguredDefault>,
    models: Vec<ModelRow>,
}

#[derive(Debug, Serialize)]
struct ModelSetOutput {
    operation: &'static str,
    provider: &'static str,
    model: &'static str,
    compatibility: &'static str,
    credential_status: &'static str,
    config_path: String,
}

pub(crate) fn list(
    provider: Option<Provider>,
    configured_only: bool,
    format: OutputFormat,
) -> Result<(), CliError> {
    let provider = provider.map(secure_credentials::canonical_provider);
    let configured = read_user_default()?;
    let configured_output = configured.as_ref().map(|(configured_provider, model)| {
        let spec = find_model(*configured_provider, model);
        ConfiguredDefault {
            provider: provider_name(*configured_provider),
            model: model.clone(),
            catalog_status: spec.map_or("unlisted", |entry| entry.compatibility.as_str()),
            credential_status: credential_status(*configured_provider),
        }
    });

    let models = MODELS
        .iter()
        .filter(|entry| provider.is_none_or(|provider| entry.provider == provider))
        .filter(|entry| {
            !configured_only
                || configured
                    .as_ref()
                    .is_some_and(|(configured_provider, model)| {
                        *configured_provider == entry.provider && model == entry.model
                    })
        })
        .map(|entry| ModelRow {
            provider: provider_name(entry.provider),
            model: entry.model,
            compatibility: entry.compatibility.as_str(),
            general_use: entry.general_use,
            recommended: entry.recommended,
            evidence: entry.evidence,
            note: entry.note,
            configured_default: configured
                .as_ref()
                .is_some_and(|(configured_provider, model)| {
                    *configured_provider == entry.provider && model == entry.model
                }),
            credential_status: credential_status(entry.provider),
        })
        .collect::<Vec<_>>();

    let output = ModelsOutput {
        catalog_version: MODEL_CATALOG_VERSION,
        configured_default: configured_output,
        models,
    };
    emit_models(&output, configured_only, format)
}

pub(crate) fn run(command: ModelCommand) -> Result<(), CliError> {
    match command {
        ModelCommand::Set {
            provider,
            model,
            format,
        } => set_default(provider, &model, format),
    }
}

fn set_default(provider: Provider, model: &str, format: OutputFormat) -> Result<(), CliError> {
    let provider = secure_credentials::canonical_provider(provider);
    let Some(spec) = find_model(provider, model) else {
        return Err(CliError::new(
            "model_unlisted",
            format!(
                "{} / {model} is not in the curated Reason model catalog; use `reason models {}` to list supported choices or pass --model explicitly for research/unlisted use",
                provider_name(provider),
                provider_name(provider)
            ),
        ));
    };
    if !spec.general_use {
        return Err(CliError::new(
            "model_not_general_use",
            format!(
                "{} / {} is cataloged as {} and cannot be saved as a general-use default; explicit --model research use remains available",
                provider_name(provider),
                spec.model,
                spec.compatibility.as_str()
            ),
        ));
    }

    let path = user_config_path().ok_or_else(|| {
        CliError::new(
            "configuration",
            "cannot determine the user config path; set REASON_HOME, XDG_CONFIG_HOME, APPDATA, or HOME",
        )
    })?;
    update_user_default(&path, provider, spec.model)?;

    let output = ModelSetOutput {
        operation: "set",
        provider: provider_name(provider),
        model: spec.model,
        compatibility: spec.compatibility.as_str(),
        credential_status: credential_status(provider),
        config_path: path.display().to_string(),
    };
    match format {
        OutputFormat::Json => print_product_json("model", &output).map_err(CliError::from),
        OutputFormat::Human => {
            println!(
                "Default model set to {} / {} ({}).",
                output.provider, output.model, output.compatibility
            );
            println!("Config: {}", output.config_path);
            if output.credential_status != "available" {
                println!("Credential status: {}", output.credential_status);
            }
            Ok(())
        }
    }
}

pub(crate) fn recommended_model(provider: Provider) -> Option<&'static str> {
    let provider = secure_credentials::canonical_provider(provider);
    MODELS
        .iter()
        .find(|e| e.provider == provider && e.general_use && e.recommended)
        .map(|e| e.model)
}

pub(crate) fn general_use_models(provider: Provider) -> Vec<(&'static str, bool, &'static str)> {
    let provider = secure_credentials::canonical_provider(provider);
    MODELS
        .iter()
        .filter(|e| e.provider == provider && e.general_use)
        .map(|e| (e.model, e.recommended, e.compatibility.as_str()))
        .collect()
}

pub(crate) fn validate_general_use_model(
    provider: Provider,
    model: &str,
) -> Result<&'static str, CliError> {
    let provider = secure_credentials::canonical_provider(provider);
    let Some(spec) = find_model(provider, model) else {
        return Err(CliError::new(
            "model_unlisted",
            format!(
                "{} / {model} is not in the curated Reason model catalog; use `reason models {}` to list supported choices",
                provider_name(provider),
                provider_name(provider)
            ),
        ));
    };
    if !spec.general_use {
        return Err(CliError::new(
            "model_not_general_use",
            format!(
                "{} / {} is cataloged as {} and cannot be used as a general-use setup default",
                provider_name(provider),
                spec.model,
                spec.compatibility.as_str()
            ),
        ));
    }
    Ok(spec.compatibility.as_str())
}

pub(crate) fn persist_user_default(provider: Provider, model: &str) -> Result<String, CliError> {
    let provider = secure_credentials::canonical_provider(provider);
    validate_general_use_model(provider, model)?;
    let path = user_config_path().ok_or_else(|| CliError::new("configuration", "cannot determine the user config path; set REASON_HOME, XDG_CONFIG_HOME, APPDATA, or HOME"))?;
    update_user_default(&path, provider, model)?;
    Ok(path.display().to_string())
}

fn find_model(provider: Provider, model: &str) -> Option<&'static ModelSpec> {
    MODELS
        .iter()
        .find(|entry| entry.provider == provider && entry.model == model)
}

fn read_user_default() -> Result<Option<(Provider, String)>, CliError> {
    let Some(path) = user_config_path() else {
        return Ok(None);
    };
    if !path.is_file() {
        return Ok(None);
    }
    let config = read_user_config(&path)?;
    Ok(match (config.run.provider, config.run.model) {
        (Some(provider), Some(model)) if !model.is_empty() => {
            Some((secure_credentials::canonical_provider(provider), model))
        }
        _ => None,
    })
}

fn read_user_config(path: &Path) -> Result<CliFileConfig, CliError> {
    if !path.is_file() {
        return Ok(CliFileConfig::default());
    }
    let bytes = fs::read(path).map_err(|error| {
        CliError::new(
            "configuration",
            format!("{}: cannot read user config: {error}", path.display()),
        )
    })?;
    let config: CliFileConfig = serde_json::from_slice(&bytes).map_err(|error| {
        CliError::new(
            "configuration",
            format!("{}: invalid user config: {error}", path.display()),
        )
    })?;
    if config.schema_version != CLI_CONFIG_CONTRACT_ID {
        return Err(CliError::new(
            "configuration",
            format!(
                "{}: unsupported config schema_version {:?}; expected {:?}",
                path.display(),
                config.schema_version,
                CLI_CONFIG_CONTRACT_ID
            ),
        ));
    }
    Ok(config)
}

fn update_user_default(path: &Path, provider: Provider, model: &str) -> Result<(), CliError> {
    let mut value = if path.is_file() {
        let bytes = fs::read(path).map_err(|error| {
            CliError::new(
                "configuration",
                format!("{}: cannot read user config: {error}", path.display()),
            )
        })?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
            CliError::new(
                "configuration",
                format!("{}: invalid user config: {error}", path.display()),
            )
        })?;
        let _: CliFileConfig = serde_json::from_value(value.clone()).map_err(|error| {
            CliError::new(
                "configuration",
                format!("{}: invalid user config: {error}", path.display()),
            )
        })?;
        value
    } else {
        serde_json::json!({
            "schema_version": CLI_CONFIG_CONTRACT_ID,
            "run": {}
        })
    };

    let object = value
        .as_object_mut()
        .ok_or_else(|| CliError::new("configuration", "user config root must be a JSON object"))?;
    let schema_version = object
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if schema_version != CLI_CONFIG_CONTRACT_ID {
        return Err(CliError::new(
            "configuration",
            format!(
                "{}: unsupported config schema_version {:?}; expected {:?}",
                path.display(),
                schema_version,
                CLI_CONFIG_CONTRACT_ID
            ),
        ));
    }
    let run = object
        .entry("run")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or_else(|| CliError::new("configuration", "user config run must be a JSON object"))?;
    run.insert(
        "provider".into(),
        serde_json::to_value(provider).map_err(|error| {
            CliError::new("configuration", format!("serialize provider: {error}"))
        })?,
    );
    run.insert("model".into(), serde_json::Value::String(model.to_string()));
    write_user_config_value(path, &value)
}

fn write_user_config_value(path: &Path, value: &serde_json::Value) -> Result<(), CliError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        local_privacy::ensure_private_directory(parent)
            .map_err(|error| CliError::new("configuration_privacy", error))?;
    }
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| {
        CliError::new(
            "configuration",
            format!("cannot serialize user config: {error}"),
        )
    })?;
    bytes.push(b'\n');
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CliError::new("configuration", "user config path must have a file name"))?;
    let temporary = path.with_file_name(format!(
        ".{file_name}.tmp-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default()
    ));

    #[cfg(unix)]
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&temporary)
            .map_err(|error| {
                CliError::new(
                    "configuration",
                    format!(
                        "{}: cannot create user config temp file: {error}",
                        temporary.display()
                    ),
                )
            })?
    };
    #[cfg(not(unix))]
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| {
            CliError::new(
                "configuration",
                format!(
                    "{}: cannot create user config temp file: {error}",
                    temporary.display()
                ),
            )
        })?;

    let result = (|| -> Result<(), CliError> {
        file.write_all(&bytes).map_err(|error| {
            CliError::new(
                "configuration",
                format!(
                    "{}: cannot write user config temp file: {error}",
                    temporary.display()
                ),
            )
        })?;
        file.sync_all().map_err(|error| {
            CliError::new(
                "configuration",
                format!(
                    "{}: cannot sync user config temp file: {error}",
                    temporary.display()
                ),
            )
        })?;
        drop(file);
        #[cfg(windows)]
        if path.exists() {
            fs::remove_file(path).map_err(|error| {
                CliError::new(
                    "configuration",
                    format!("{}: cannot replace user config: {error}", path.display()),
                )
            })?;
        }
        fs::rename(&temporary, path).map_err(|error| {
            CliError::new(
                "configuration",
                format!("{}: cannot commit user config: {error}", path.display()),
            )
        })?;
        local_privacy::ensure_private_file(path)
            .map_err(|error| CliError::new("configuration_privacy", error))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn credential_status(provider: Provider) -> &'static str {
    match secure_credentials::resolve_provider_credential(provider) {
        Ok(_) => "available",
        Err(error) => match error.kind {
            secure_credentials::CredentialErrorKind::Missing => "missing",
            secure_credentials::CredentialErrorKind::InvalidEnvironment => "invalid_environment",
            secure_credentials::CredentialErrorKind::StoreUnavailable => {
                "credential_store_unavailable"
            }
            secure_credentials::CredentialErrorKind::StoreFailure => "credential_store_error",
            secure_credentials::CredentialErrorKind::InvalidSecret => "invalid_os_store",
        },
    }
}

fn emit_models(
    output: &ModelsOutput,
    configured_only: bool,
    format: OutputFormat,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Json => print_product_json("models", output).map_err(CliError::from),
        OutputFormat::Human => {
            if let Some(default) = &output.configured_default {
                println!(
                    "Configured default: {} / {} (catalog={}, credential={})",
                    default.provider,
                    default.model,
                    default.catalog_status,
                    default.credential_status
                );
            } else {
                println!("Configured default: none");
            }
            if configured_only && output.models.is_empty() {
                if output.configured_default.is_some() {
                    println!("The configured model is not in the curated catalog.");
                }
                return Ok(());
            }
            for row in &output.models {
                let marker = if row.configured_default {
                    " [default]"
                } else if row.recommended {
                    " [recommended]"
                } else {
                    ""
                };
                println!(
                    "{} / {}{} — {}{}",
                    row.provider,
                    row.model,
                    marker,
                    row.compatibility,
                    if row.general_use {
                        ""
                    } else {
                        ", research-only"
                    }
                );
                println!("  evidence={} — {}", row.evidence, row.note);
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_one_recommended_general_use_model_for_primary_providers() {
        for provider in [Provider::Mistral, Provider::Google, Provider::Groq] {
            let recommended = MODELS
                .iter()
                .filter(|entry| {
                    entry.provider == provider && entry.general_use && entry.recommended
                })
                .collect::<Vec<_>>();
            assert_eq!(recommended.len(), 1, "provider={}", provider_name(provider));
        }
    }

    #[test]
    fn nvidia_negative_control_is_visible_but_not_general_use() {
        let model = find_model(Provider::Nvidia, "nvidia/nemotron-3.5-lightning-30b-a3b").unwrap();
        assert_eq!(model.compatibility, Compatibility::Limited);
        assert!(!model.general_use);
        assert!(!model.recommended);
    }
}
