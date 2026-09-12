use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Subcommand, ValueEnum};
use serde::Serialize;

use super::{
    CLI_CONFIG_CONTRACT_ID, CliError, CliFileConfig, OutputFormat, Provider, load_cli_config,
    model_catalog, print_product_json, project_config_path, provider_name, user_config_path,
};

const SURFACE_ID: &str = "reason-config-surface-v1";
const KEYS: &[&str] = &[
    "run.provider",
    "run.model",
    "run.max_tokens",
    "run.max_model_calls",
    "run.max_output_tokens",
    "run.max_total_tokens",
    "run.format",
];

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigCommand {
    List {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    Get {
        key: String,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    Set {
        key: String,
        value: String,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    Unset {
        key: String,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    Path {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    Sources {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
}

impl ConfigCommand {
    pub(crate) const fn format(&self) -> OutputFormat {
        match self {
            Self::List { format, .. }
            | Self::Get { format, .. }
            | Self::Set { format, .. }
            | Self::Unset { format, .. }
            | Self::Path { format, .. }
            | Self::Sources { format, .. } => *format,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct Entry {
    key: &'static str,
    value: serde_json::Value,
    source: &'static str,
    editable: bool,
}
#[derive(Debug, Serialize)]
struct ListOutput {
    config_surface: &'static str,
    config_contract: &'static str,
    entries: Vec<Entry>,
}
#[derive(Debug, Serialize)]
struct GetOutput {
    config_surface: &'static str,
    config_contract: &'static str,
    entry: Entry,
}
#[derive(Debug, Serialize)]
struct MutationOutput {
    config_surface: &'static str,
    config_contract: &'static str,
    operation: &'static str,
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<serde_json::Value>,
    config_path: String,
}
#[derive(Debug, Serialize)]
struct PathOutput {
    config_surface: &'static str,
    user: Option<String>,
    project: Option<String>,
    explicit: Option<String>,
}
#[derive(Debug, Serialize)]
struct SourceRow {
    precedence: u8,
    source: &'static str,
    path: Option<String>,
    present: bool,
    note: &'static str,
}
#[derive(Debug, Serialize)]
struct SourcesOutput {
    config_surface: &'static str,
    precedence_low_to_high: Vec<SourceRow>,
    effective: Vec<Entry>,
}

pub(crate) fn run(command: ConfigCommand) -> Result<(), CliError> {
    match command {
        ConfigCommand::List { config, format } => emit_list(config.as_ref(), format),
        ConfigCommand::Get {
            key,
            config,
            format,
        } => emit_get(&key, config.as_ref(), format),
        ConfigCommand::Set { key, value, format } => set_user(&key, &value, format),
        ConfigCommand::Unset { key, format } => unset_user(&key, format),
        ConfigCommand::Path { config, format } => emit_paths(config.as_ref(), format),
        ConfigCommand::Sources { config, format } => emit_sources(config.as_ref(), format),
    }
}

fn require_key(key: &str) -> Result<&'static str, CliError> {
    KEYS.iter().copied().find(|k| *k == key).ok_or_else(|| {
        let lower = key.to_ascii_lowercase();
        if ["key", "token", "secret", "password", "credential"]
            .iter()
            .any(|v| lower.contains(v))
        {
            CliError::new(
                "configuration_secret",
                format!(
                    "{key:?} is not a non-secret config key; use `reason auth` for credentials"
                ),
            )
        } else {
            CliError::new(
                "configuration_key",
                format!(
                    "unsupported config key {key:?}; editable keys: {}",
                    KEYS.join(", ")
                ),
            )
        }
    })
}

fn load_effective(explicit: Option<&PathBuf>) -> Result<Vec<Entry>, CliError> {
    let loaded = load_cli_config(explicit).map_err(|e| CliError::new("configuration", e))?;
    let origins = origins(explicit)?;
    Ok(KEYS
        .iter()
        .map(|key| Entry {
            key,
            value: value_of(&loaded.config, key),
            source: origins
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, s)| *s)
                .unwrap_or("default"),
            editable: true,
        })
        .collect())
}

fn read_config(path: &Path) -> Result<CliFileConfig, CliError> {
    let bytes = fs::read(path)
        .map_err(|e| CliError::new("configuration", format!("{}: {e}", path.display())))?;
    let cfg: CliFileConfig = serde_json::from_slice(&bytes)
        .map_err(|e| CliError::new("configuration", format!("{}: {e}", path.display())))?;
    if cfg.schema_version != CLI_CONFIG_CONTRACT_ID {
        return Err(CliError::new(
            "configuration",
            format!(
                "{}: unsupported schema_version {:?}; expected {:?}",
                path.display(),
                cfg.schema_version,
                CLI_CONFIG_CONTRACT_ID
            ),
        ));
    }
    Ok(cfg)
}

fn origins(explicit: Option<&PathBuf>) -> Result<Vec<(&'static str, &'static str)>, CliError> {
    let mut out = Vec::new();
    if let Some(p) = user_config_path().filter(|p| p.is_file()) {
        record_origins(&mut out, &read_config(&p)?, "user");
    }
    if let Some(p) = project_config_path().filter(|p| p.is_file()) {
        record_origins(&mut out, &read_config(&p)?, "project");
    }
    if let Some(p) = explicit {
        record_origins(&mut out, &read_config(p)?, "explicit");
    }
    Ok(out)
}

fn record_origins(
    out: &mut Vec<(&'static str, &'static str)>,
    cfg: &CliFileConfig,
    source: &'static str,
) {
    for key in KEYS {
        if value_of(cfg, key) != serde_json::Value::Null {
            if let Some(row) = out.iter_mut().find(|(k, _)| k == key) {
                row.1 = source;
            } else {
                out.push((key, source));
            }
        }
    }
}

fn value_of(cfg: &CliFileConfig, key: &str) -> serde_json::Value {
    match key {
        "run.provider" => cfg
            .run
            .provider
            .map(|p| serde_json::json!(provider_name(p)))
            .unwrap_or(serde_json::Value::Null),
        "run.model" => cfg
            .run
            .model
            .clone()
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null),
        "run.max_tokens" => cfg
            .run
            .max_tokens
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null),
        "run.max_model_calls" => cfg
            .run
            .max_model_calls
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null),
        "run.max_output_tokens" => cfg
            .run
            .max_output_tokens
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null),
        "run.max_total_tokens" => cfg
            .run
            .max_total_tokens
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null),
        "run.format" => cfg
            .run
            .format
            .map(|f| {
                serde_json::json!(match f {
                    OutputFormat::Human => "human",
                    OutputFormat::Json => "json",
                })
            })
            .unwrap_or(serde_json::Value::Null),
        _ => serde_json::Value::Null,
    }
}

fn parse_value(key: &str, raw: &str) -> Result<serde_json::Value, CliError> {
    match key {
        "run.provider" => Provider::from_str(raw, true)
            .map(|p| serde_json::json!(provider_name(p)))
            .map_err(|_| {
                CliError::new(
                    "configuration_value",
                    "run.provider must be mistral, google, groq, or nvidia",
                )
            }),
        "run.model" if raw.trim().is_empty() => Err(CliError::new(
            "configuration_value",
            "run.model must not be empty",
        )),
        "run.model" => Ok(serde_json::json!(raw)),
        "run.format" => OutputFormat::from_str(raw, true)
            .map(|f| {
                serde_json::json!(match f {
                    OutputFormat::Human => "human",
                    OutputFormat::Json => "json",
                })
            })
            .map_err(|_| CliError::new("configuration_value", "run.format must be human or json")),
        "run.max_tokens" => {
            let n = positive(raw, key)?;
            let n = u32::try_from(n).map_err(|_| {
                CliError::new("configuration_value", "run.max_tokens exceeds u32 range")
            })?;
            Ok(serde_json::json!(n))
        }
        "run.max_model_calls" | "run.max_output_tokens" | "run.max_total_tokens" => {
            Ok(serde_json::json!(positive(raw, key)?))
        }
        _ => Err(CliError::new("configuration_key", "unsupported config key")),
    }
}

fn positive(raw: &str, key: &str) -> Result<u64, CliError> {
    let n = raw.parse::<u64>().map_err(|_| {
        CliError::new(
            "configuration_value",
            format!("{key} must be a positive integer"),
        )
    })?;
    if n == 0 {
        Err(CliError::new(
            "configuration_value",
            format!("{key} must be at least 1"),
        ))
    } else {
        Ok(n)
    }
}

fn user_json() -> Result<(PathBuf, serde_json::Value), CliError> {
    let path = user_config_path().ok_or_else(|| {
        CliError::new(
            "configuration",
            "cannot determine user config path; set REASON_HOME, XDG_CONFIG_HOME, APPDATA, or HOME",
        )
    })?;
    if !path.is_file() {
        return Ok((
            path,
            serde_json::json!({"schema_version":CLI_CONFIG_CONTRACT_ID,"run":{}}),
        ));
    }
    let bytes = fs::read(&path)
        .map_err(|e| CliError::new("configuration", format!("{}: {e}", path.display())))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| CliError::new("configuration", format!("{}: {e}", path.display())))?;
    let cfg: CliFileConfig = serde_json::from_value(value.clone())
        .map_err(|e| CliError::new("configuration", format!("{}: {e}", path.display())))?;
    if cfg.schema_version != CLI_CONFIG_CONTRACT_ID {
        return Err(CliError::new(
            "configuration",
            format!("{}: unsupported schema_version", path.display()),
        ));
    }
    Ok((path, value))
}

fn mutate_user(key: &str, value: Option<serde_json::Value>) -> Result<PathBuf, CliError> {
    let (path, mut root) = user_json()?;
    let obj = root
        .as_object_mut()
        .ok_or_else(|| CliError::new("configuration", "user config root must be a JSON object"))?;
    let run = obj
        .entry("run")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or_else(|| CliError::new("configuration", "user config run must be an object"))?;
    let leaf = key.strip_prefix("run.").expect("safe keys are run.*");
    if let Some(v) = value {
        run.insert(leaf.into(), v);
    } else {
        run.remove(leaf);
    }
    let parsed: CliFileConfig = serde_json::from_value(root.clone()).map_err(|e| {
        CliError::new(
            "configuration_value",
            format!("resulting config invalid: {e}"),
        )
    })?;
    validate_config(&parsed)?;
    model_catalog::write_user_config_value(&path, &root)?;
    Ok(path)
}

fn validate_config(cfg: &CliFileConfig) -> Result<(), CliError> {
    if cfg.run.max_tokens == Some(0) {
        return Err(CliError::new(
            "configuration_value",
            "run.max_tokens must be at least 1",
        ));
    }
    for (k, v) in [
        ("run.max_model_calls", cfg.run.max_model_calls),
        ("run.max_output_tokens", cfg.run.max_output_tokens),
        ("run.max_total_tokens", cfg.run.max_total_tokens),
    ] {
        if v == Some(0) {
            return Err(CliError::new(
                "configuration_value",
                format!("{k} must be at least 1"),
            ));
        }
    }
    if let (Some(provider), Some(model)) = (cfg.run.provider, cfg.run.model.as_deref()) {
        model_catalog::validate_general_use_model(provider, model).map(|_| ())?;
    }
    Ok(())
}

fn set_user(key: &str, raw: &str, format: OutputFormat) -> Result<(), CliError> {
    let key = require_key(key)?;
    let value = parse_value(key, raw)?;
    let path = mutate_user(key, Some(value.clone()))?;
    emit_mutation("set", key, Some(value), path, format)
}
fn unset_user(key: &str, format: OutputFormat) -> Result<(), CliError> {
    let key = require_key(key)?;
    let path = mutate_user(key, None)?;
    emit_mutation("unset", key, None, path, format)
}
fn emit_mutation(
    op: &'static str,
    key: &str,
    value: Option<serde_json::Value>,
    path: PathBuf,
    format: OutputFormat,
) -> Result<(), CliError> {
    let out = MutationOutput {
        config_surface: SURFACE_ID,
        config_contract: CLI_CONFIG_CONTRACT_ID,
        operation: op,
        key: key.into(),
        value,
        config_path: path.display().to_string(),
    };
    match format {
        OutputFormat::Json => print_product_json("config", &out).map_err(CliError::from),
        OutputFormat::Human => {
            if let Some(v) = &out.value {
                println!("Set {}={}.", out.key, human(v));
            } else {
                println!("Unset {}.", out.key);
            }
            println!("Config: {}", out.config_path);
            Ok(())
        }
    }
}

fn emit_list(explicit: Option<&PathBuf>, format: OutputFormat) -> Result<(), CliError> {
    let out = ListOutput {
        config_surface: SURFACE_ID,
        config_contract: CLI_CONFIG_CONTRACT_ID,
        entries: load_effective(explicit)?,
    };
    match format {
        OutputFormat::Json => print_product_json("config", &out).map_err(CliError::from),
        OutputFormat::Human => {
            for e in out.entries {
                println!("{}={}  source={}", e.key, human(&e.value), e.source);
            }
            Ok(())
        }
    }
}
fn emit_get(key: &str, explicit: Option<&PathBuf>, format: OutputFormat) -> Result<(), CliError> {
    let key = require_key(key)?;
    let entry = load_effective(explicit)?
        .into_iter()
        .find(|e| e.key == key)
        .expect("known key");
    let out = GetOutput {
        config_surface: SURFACE_ID,
        config_contract: CLI_CONFIG_CONTRACT_ID,
        entry,
    };
    match format {
        OutputFormat::Json => print_product_json("config", &out).map_err(CliError::from),
        OutputFormat::Human => {
            println!(
                "{}={}  source={}",
                out.entry.key,
                human(&out.entry.value),
                out.entry.source
            );
            Ok(())
        }
    }
}
fn emit_paths(explicit: Option<&PathBuf>, format: OutputFormat) -> Result<(), CliError> {
    let out = PathOutput {
        config_surface: SURFACE_ID,
        user: user_config_path().map(|p| p.display().to_string()),
        project: project_config_path().map(|p| p.display().to_string()),
        explicit: explicit.map(|p| p.display().to_string()),
    };
    match format {
        OutputFormat::Json => print_product_json("config", &out).map_err(CliError::from),
        OutputFormat::Human => {
            println!("user: {}", out.user.as_deref().unwrap_or("unavailable"));
            println!(
                "project: {}",
                out.project.as_deref().unwrap_or("unavailable")
            );
            println!("explicit: {}", out.explicit.as_deref().unwrap_or("none"));
            Ok(())
        }
    }
}

fn emit_sources(explicit: Option<&PathBuf>, format: OutputFormat) -> Result<(), CliError> {
    let effective = load_effective(explicit)?;
    let user = user_config_path();
    let project = project_config_path();
    let rows = vec![
        SourceRow {
            precedence: 0,
            source: "default",
            path: None,
            present: true,
            note: "built-in defaults",
        },
        SourceRow {
            precedence: 1,
            source: "user",
            present: user.as_ref().is_some_and(|p| p.is_file()),
            path: user.map(|p| p.display().to_string()),
            note: "persistent user config; config set/unset edits this layer",
        },
        SourceRow {
            precedence: 2,
            source: "project",
            present: project.as_ref().is_some_and(|p| p.is_file()),
            path: project.map(|p| p.display().to_string()),
            note: "project config; executable/acquisition settings remain subject to project trust",
        },
        SourceRow {
            precedence: 3,
            source: "explicit",
            present: explicit.is_some_and(|p| p.is_file()),
            path: explicit.map(|p| p.display().to_string()),
            note: "explicit --config layer for this inspection/invocation",
        },
        SourceRow {
            precedence: 4,
            source: "environment",
            path: None,
            present: false,
            note: "no non-secret run-default env layer; credentials are separate auth inputs",
        },
        SourceRow {
            precedence: 5,
            source: "cli",
            path: None,
            present: false,
            note: "per-invocation flags override persisted config and are not persisted here",
        },
    ];
    let out = SourcesOutput {
        config_surface: SURFACE_ID,
        precedence_low_to_high: rows,
        effective,
    };
    match format {
        OutputFormat::Json => print_product_json("config", &out).map_err(CliError::from),
        OutputFormat::Human => {
            println!("Precedence (low -> high):");
            for r in &out.precedence_low_to_high {
                println!(
                    "  {}. {}  present={}{}",
                    r.precedence,
                    r.source,
                    r.present,
                    r.path
                        .as_deref()
                        .map(|p| format!("  {p}"))
                        .unwrap_or_default()
                );
                println!("     {}", r.note);
            }
            println!("Effective safe run config:");
            for e in out.effective {
                println!("  {}={}  source={}", e.key, human(&e.value), e.source);
            }
            Ok(())
        }
    }
}
fn human(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "<unset>".into(),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn surface_rejects_secrets_and_authority() {
        assert!(require_key("run.provider").is_ok());
        assert!(require_key("run.max_model_calls").is_ok());
        assert!(require_key("api_key").is_err());
        assert!(require_key("resolution.trusted_command.program").is_err());
    }
    #[test]
    fn typed_values_validate() {
        assert_eq!(
            parse_value("run.max_tokens", "64").unwrap(),
            serde_json::json!(64)
        );
        assert!(parse_value("run.max_tokens", "0").is_err());
        assert_eq!(
            parse_value("run.format", "json").unwrap(),
            serde_json::json!("json")
        );
        assert!(parse_value("run.provider", "bogus").is_err());
    }
}
