use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    CLI_CONFIG_CONTRACT_ID, CliFileConfig, InvestigationCapabilityFileConfig, LoadedCliConfig,
    ResolutionFileConfig, local_privacy, merge_cli_config, user_config_path,
};

pub(crate) const PROJECT_TRUST_CONTRACT_ID: &str = "reason-project-trust-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectTrustRecord {
    config_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectTrustStore {
    schema_version: String,
    #[serde(default)]
    projects: BTreeMap<String, ProjectTrustRecord>,
}

impl Default for ProjectTrustStore {
    fn default() -> Self {
        Self {
            schema_version: PROJECT_TRUST_CONTRACT_ID.into(),
            projects: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectExecutableIdentity {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectTrustStatus {
    pub contract_id: &'static str,
    pub project_path: String,
    pub config_path: String,
    pub config_exists: bool,
    pub high_risk_config: bool,
    pub trusted: bool,
    pub state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stored_fingerprint: Option<String>,
    pub executable_programs: Vec<String>,
    pub executable_identities: Vec<ProjectExecutableIdentity>,
    pub project_trusted_command_allowed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StoredProjectTrust {
    pub project_path: String,
    pub config_fingerprint: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectTrustList {
    pub contract_id: &'static str,
    pub projects: Vec<StoredProjectTrust>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectTrustRevocation {
    pub contract_id: &'static str,
    pub revoked: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_path: Option<String>,
    pub all: bool,
}

struct ProjectInspection {
    project_root: PathBuf,
    project_key: String,
    config_path: PathBuf,
    config_exists: bool,
    overlay: Option<CliFileConfig>,
    high_risk: bool,
    fingerprint: Option<String>,
    executable_programs: Vec<String>,
    executable_identities: Vec<ProjectExecutableIdentity>,
    trusted_command_present: bool,
}

pub(crate) fn merge_project_config(
    loaded: &mut LoadedCliConfig,
    config_path: &Path,
) -> Result<(), String> {
    let project_root = config_path
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| format!("{}: invalid project config path", config_path.display()))?;
    let inspection = inspect_project(project_root)?;
    let Some(overlay) = inspection.overlay else {
        return Ok(());
    };

    if inspection.trusted_command_present {
        return Err(format!(
            "{}: resolution.trusted_command is forbidden in project config even when the folder is trusted; place hard-verifier authority in user config or pass an explicit --config file",
            inspection.config_path.display()
        ));
    }

    if inspection.high_risk {
        let store = load_store()?;
        let record = store.projects.get(&inspection.project_key);
        let current = inspection
            .fingerprint
            .as_deref()
            .expect("high-risk project config has a fingerprint");
        match record {
            Some(record) if record.config_fingerprint == current => {}
            Some(record) => {
                return Err(format!(
                    "{}: project trust is stale because executable/acquisition settings changed (stored {}, current {}); review the config and run `reason trust add` again, or use --no-config",
                    inspection.config_path.display(),
                    short_fingerprint(&record.config_fingerprint),
                    short_fingerprint(current),
                ));
            }
            None => {
                return Err(format!(
                    "{}: project config contains executable/network acquisition settings but this exact project/config is not trusted; review it and run `reason trust add`, or use --no-config (fingerprint {})",
                    inspection.config_path.display(),
                    short_fingerprint(current),
                ));
            }
        }
    }

    merge_cli_config(&mut loaded.config, overlay);
    loaded.sources.push("project");
    Ok(())
}

pub(crate) fn status(project: Option<&Path>) -> Result<ProjectTrustStatus, String> {
    let root = resolve_project_root(project)?;
    let inspection = inspect_project(&root)?;
    status_from_inspection(&inspection)
}

pub(crate) fn add(project: Option<&Path>) -> Result<ProjectTrustStatus, String> {
    let root = resolve_project_root(project)?;
    let inspection = inspect_project(&root)?;
    if !inspection.config_exists {
        return Err(format!(
            "{}: no .reason/config.json exists to trust",
            inspection.project_root.display()
        ));
    }
    if inspection.trusted_command_present {
        return Err(format!(
            "{}: resolution.trusted_command cannot be trusted from project config; use user config or an explicit --config file for hard-verifier authority",
            inspection.config_path.display()
        ));
    }
    if !inspection.high_risk {
        return status_from_inspection(&inspection);
    }
    let fingerprint = inspection
        .fingerprint
        .clone()
        .expect("high-risk project config has a fingerprint");
    let mut store = load_store()?;
    store.projects.insert(
        inspection.project_key.clone(),
        ProjectTrustRecord {
            config_fingerprint: fingerprint,
        },
    );
    write_store(&store)?;
    status_from_inspection(&inspection)
}

pub(crate) fn revoke(project: Option<&Path>, all: bool) -> Result<ProjectTrustRevocation, String> {
    let mut store = load_store()?;
    if all {
        let revoked = store.projects.len();
        store.projects.clear();
        write_store(&store)?;
        return Ok(ProjectTrustRevocation {
            contract_id: PROJECT_TRUST_CONTRACT_ID,
            revoked,
            project_path: None,
            all: true,
        });
    }

    let root = resolve_project_root(project)?;
    let key = path_key(&root)?;
    let revoked = usize::from(store.projects.remove(&key).is_some());
    write_store(&store)?;
    Ok(ProjectTrustRevocation {
        contract_id: PROJECT_TRUST_CONTRACT_ID,
        revoked,
        project_path: Some(key),
        all: false,
    })
}

pub(crate) fn list() -> Result<ProjectTrustList, String> {
    let store = load_store()?;
    Ok(ProjectTrustList {
        contract_id: PROJECT_TRUST_CONTRACT_ID,
        projects: store
            .projects
            .into_iter()
            .map(|(project_path, record)| StoredProjectTrust {
                project_path,
                config_fingerprint: record.config_fingerprint,
            })
            .collect(),
    })
}

fn status_from_inspection(inspection: &ProjectInspection) -> Result<ProjectTrustStatus, String> {
    let store = load_store()?;
    let stored = store.projects.get(&inspection.project_key);
    let trusted = inspection.high_risk
        && !inspection.trusted_command_present
        && stored.is_some_and(|record| {
            inspection.fingerprint.as_deref() == Some(record.config_fingerprint.as_str())
        });
    let state = if !inspection.config_exists {
        "missing_config"
    } else if inspection.trusted_command_present {
        "forbidden_authority"
    } else if !inspection.high_risk {
        "not_required"
    } else if trusted {
        "trusted"
    } else if stored.is_some() {
        "stale"
    } else {
        "untrusted"
    };
    Ok(ProjectTrustStatus {
        contract_id: PROJECT_TRUST_CONTRACT_ID,
        project_path: inspection.project_key.clone(),
        config_path: path_key_lossless(&inspection.config_path)?,
        config_exists: inspection.config_exists,
        high_risk_config: inspection.high_risk,
        trusted,
        state,
        current_fingerprint: inspection.fingerprint.clone(),
        stored_fingerprint: stored.map(|record| record.config_fingerprint.clone()),
        executable_programs: inspection.executable_programs.clone(),
        executable_identities: inspection.executable_identities.clone(),
        project_trusted_command_allowed: false,
    })
}

fn inspect_project(project_root: &Path) -> Result<ProjectInspection, String> {
    let project_root = fs::canonicalize(project_root).map_err(|error| {
        format!(
            "{}: cannot resolve project path: {error}",
            project_root.display()
        )
    })?;
    if !project_root.is_dir() {
        return Err(format!(
            "{}: project path is not a directory",
            project_root.display()
        ));
    }
    let project_key = path_key_lossless(&project_root)?;
    let config_path = project_root.join(".reason").join("config.json");
    if !config_path.is_file() {
        return Ok(ProjectInspection {
            project_root,
            project_key,
            config_path,
            config_exists: false,
            overlay: None,
            high_risk: false,
            fingerprint: None,
            executable_programs: vec![],
            executable_identities: vec![],
            trusted_command_present: false,
        });
    }

    let canonical_config = fs::canonicalize(&config_path).map_err(|error| {
        format!(
            "{}: cannot resolve project config: {error}",
            config_path.display()
        )
    })?;
    if !canonical_config.starts_with(&project_root) {
        return Err(format!(
            "{}: project config resolves outside the canonical project root; refusing symlink/path escape",
            config_path.display()
        ));
    }
    let bytes = fs::read(&canonical_config)
        .map_err(|error| format!("{}: {error}", canonical_config.display()))?;
    let mut overlay: CliFileConfig = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{}: {error}", canonical_config.display()))?;
    if overlay.schema_version != CLI_CONFIG_CONTRACT_ID {
        return Err(format!(
            "{}: unsupported config schema_version {:?}; expected {:?}",
            canonical_config.display(),
            overlay.schema_version,
            CLI_CONFIG_CONTRACT_ID
        ));
    }

    let trusted_command_present = overlay.resolution.trusted_command.is_some();
    let high_risk = has_high_risk_resolution(&overlay.resolution);
    let mut executable_programs = vec![];
    if high_risk {
        normalize_project_executables(
            &project_root,
            &mut overlay.resolution,
            &mut executable_programs,
        )?;
    }
    let executable_identities = executable_programs
        .iter()
        .map(|program| executable_identity(Path::new(program)))
        .collect::<Result<Vec<_>, _>>()?;
    let fingerprint = high_risk
        .then(|| fingerprint_resolution(&overlay.resolution, &executable_identities))
        .transpose()?;

    Ok(ProjectInspection {
        project_root,
        project_key,
        config_path: canonical_config,
        config_exists: true,
        overlay: Some(overlay),
        high_risk,
        fingerprint,
        executable_programs,
        executable_identities,
        trusted_command_present,
    })
}

fn resolve_project_root(project: Option<&Path>) -> Result<PathBuf, String> {
    let requested = match project {
        Some(path) => path.to_path_buf(),
        None => {
            env::current_dir().map_err(|error| format!("cannot read current directory: {error}"))?
        }
    };
    fs::canonicalize(&requested).map_err(|error| {
        format!(
            "{}: cannot resolve project path: {error}",
            requested.display()
        )
    })
}

fn has_high_risk_resolution(resolution: &ResolutionFileConfig) -> bool {
    resolution.external_command.is_some()
        || resolution.mcp_readonly.is_some()
        || resolution.trusted_command.is_some()
        || resolution.investigation.is_some()
}

fn normalize_project_executables(
    project_root: &Path,
    resolution: &mut ResolutionFileConfig,
    programs: &mut Vec<String>,
) -> Result<(), String> {
    if let Some(config) = resolution.external_command.as_mut() {
        config.program = resolve_program(project_root, &config.program)?;
        programs.push(config.program.clone());
    }
    if let Some(config) = resolution.mcp_readonly.as_mut() {
        config.program = resolve_program(project_root, &config.program)?;
        programs.push(config.program.clone());
    }
    if let Some(config) = resolution.trusted_command.as_mut() {
        config.program = resolve_program(project_root, &config.program)?;
        programs.push(config.program.clone());
    }
    if let Some(investigation) = resolution.investigation.as_mut() {
        for capability in &mut investigation.capabilities {
            match capability {
                InvestigationCapabilityFileConfig::ExternalCommand { program, .. }
                | InvestigationCapabilityFileConfig::McpReadonly { program, .. } => {
                    *program = resolve_program(project_root, program)?;
                    programs.push(program.clone());
                }
            }
        }
    }
    programs.sort();
    programs.dedup();
    Ok(())
}

fn resolve_program(project_root: &Path, program: &str) -> Result<String, String> {
    if program.trim().is_empty() {
        return Err("project executable program must not be empty".into());
    }
    let configured = Path::new(program);
    let candidate = if configured.is_absolute() {
        configured.to_path_buf()
    } else if configured.components().count() > 1 {
        project_root.join(configured)
    } else {
        resolve_bare_program(program)?
    };
    let canonical = fs::canonicalize(&candidate).map_err(|error| {
        format!(
            "project executable {program:?} could not be resolved to a stable executable path: {error}"
        )
    })?;
    if !canonical.is_file() {
        return Err(format!(
            "project executable {:?} resolves to non-file {}",
            program,
            canonical.display()
        ));
    }
    if !configured.is_absolute()
        && configured.components().count() > 1
        && !canonical.starts_with(project_root)
    {
        return Err(format!(
            "project-relative executable {:?} resolves outside the project root: {}",
            program,
            canonical.display()
        ));
    }
    path_key_lossless(&canonical)
}

fn resolve_bare_program(program: &str) -> Result<PathBuf, String> {
    let path = env::var_os("PATH").ok_or_else(|| {
        format!("PATH is unavailable while resolving project executable {program:?}")
    })?;
    for directory in env::split_paths(&path) {
        for candidate in path_candidates(&directory, program) {
            if candidate.is_file() && executable_candidate(&candidate) {
                return Ok(candidate);
            }
        }
    }
    Err(format!(
        "project executable {program:?} was not found on PATH; use an absolute or project-relative executable path if the identity must be explicit"
    ))
}

#[cfg(windows)]
fn path_candidates(directory: &Path, program: &str) -> Vec<PathBuf> {
    let configured = Path::new(program);
    if configured.extension().is_some() {
        return vec![directory.join(configured)];
    }
    let path_ext = env::var_os("PATHEXT")
        .unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into())
        .to_string_lossy()
        .to_string();
    path_ext
        .split(';')
        .filter(|extension| !extension.is_empty())
        .map(|extension| directory.join(format!("{program}{extension}")))
        .collect()
}

#[cfg(not(windows))]
fn path_candidates(directory: &Path, program: &str) -> Vec<PathBuf> {
    vec![directory.join(program)]
}

#[cfg(unix)]
fn executable_candidate(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn executable_candidate(_path: &Path) -> bool {
    true
}

fn executable_identity(path: &Path) -> Result<ProjectExecutableIdentity, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "{}: cannot hash project executable identity: {error}",
            path.display()
        )
    })?;
    let digest = Sha256::digest(bytes);
    Ok(ProjectExecutableIdentity {
        path: path_key_lossless(path)?,
        sha256: format!("sha256:{}", hex_lower(&digest)),
    })
}

fn fingerprint_resolution(
    resolution: &ResolutionFileConfig,
    executable_identities: &[ProjectExecutableIdentity],
) -> Result<String, String> {
    #[derive(Serialize)]
    struct Material<'a> {
        schema_version: &'a str,
        resolution: &'a ResolutionFileConfig,
        executable_identities: &'a [ProjectExecutableIdentity],
    }
    let bytes = serde_json::to_vec(&Material {
        schema_version: CLI_CONFIG_CONTRACT_ID,
        resolution,
        executable_identities,
    })
    .map_err(|error| format!("cannot serialize project trust fingerprint material: {error}"))?;
    let digest = Sha256::digest(bytes);
    Ok(format!("sha256:{}", hex_lower(&digest)))
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("write to String cannot fail");
    }
    output
}

fn short_fingerprint(fingerprint: &str) -> &str {
    fingerprint.get(..19).unwrap_or(fingerprint)
}

fn trust_store_path() -> Result<PathBuf, String> {
    let config = user_config_path().ok_or_else(|| {
        "cannot determine Reason user config directory; set REASON_HOME or a supported user home/config environment variable".to_string()
    })?;
    let directory = config
        .parent()
        .ok_or_else(|| format!("{}: invalid user config path", config.display()))?;
    Ok(directory.join("project-trust.json"))
}

fn load_store() -> Result<ProjectTrustStore, String> {
    let path = trust_store_path()?;
    if !path.exists() {
        return Ok(ProjectTrustStore::default());
    }
    let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let store: ProjectTrustStore = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{}: invalid project trust store: {error}", path.display()))?;
    if store.schema_version != PROJECT_TRUST_CONTRACT_ID {
        return Err(format!(
            "{}: unsupported project trust schema {:?}; expected {:?}",
            path.display(),
            store.schema_version,
            PROJECT_TRUST_CONTRACT_ID
        ));
    }
    Ok(store)
}

fn write_store(store: &ProjectTrustStore) -> Result<(), String> {
    let path = trust_store_path()?;
    let directory = path
        .parent()
        .ok_or_else(|| format!("{}: invalid project trust path", path.display()))?;
    local_privacy::ensure_private_directory(directory)?;
    let mut bytes = serde_json::to_vec_pretty(store)
        .map_err(|error| format!("cannot serialize project trust store: {error}"))?;
    bytes.push(b'\n');
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{}: invalid project trust file name", path.display()))?;
    let temporary = directory.join(format!(".{file_name}.tmp-{}", std::process::id()));
    let _ = fs::remove_file(&temporary);

    #[cfg(unix)]
    {
        use std::io::Write as _;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&temporary)
            .map_err(|error| {
                format!(
                    "{}: cannot create trust temp file: {error}",
                    temporary.display()
                )
            })?;
        file.write_all(&bytes).map_err(|error| {
            format!(
                "{}: cannot write trust temp file: {error}",
                temporary.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "{}: cannot sync trust temp file: {error}",
                temporary.display()
            )
        })?;
    }
    #[cfg(not(unix))]
    {
        use std::io::Write as _;
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|error| {
                format!(
                    "{}: cannot create trust temp file: {error}",
                    temporary.display()
                )
            })?;
        file.write_all(&bytes).map_err(|error| {
            format!(
                "{}: cannot write trust temp file: {error}",
                temporary.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "{}: cannot sync trust temp file: {error}",
                temporary.display()
            )
        })?;
    }

    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|error| format!("{}: cannot replace trust store: {error}", path.display()))?;
    }
    fs::rename(&temporary, &path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("{}: cannot commit trust store: {error}", path.display())
    })?;
    local_privacy::ensure_private_file(&path)?;
    Ok(())
}

fn path_key(path: &Path) -> Result<String, String> {
    let canonical = fs::canonicalize(path)
        .map_err(|error| format!("{}: cannot resolve canonical path: {error}", path.display()))?;
    path_key_lossless(&canonical)
}

fn path_key_lossless(path: &Path) -> Result<String, String> {
    path.to_str().map(str::to_string).ok_or_else(|| {
        format!(
            "{}: project trust requires a UTF-8 canonical path",
            path.display()
        )
    })
}
