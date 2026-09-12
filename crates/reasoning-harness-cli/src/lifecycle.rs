use std::{
    env, fs,
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Args;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    CliError, OutputFormat, Provider, print_product_json, secure_credentials, user_config_path,
};

const REPOSITORY: &str = "git-ksk/reasoning-harness";
const REPOSITORY_URL: &str = "https://github.com/git-ksk/reasoning-harness";
const API_RELEASES_URL: &str =
    "https://api.github.com/repos/git-ksk/reasoning-harness/releases?per_page=100";
const SIGNER_WORKFLOW: &str = "git-ksk/reasoning-harness/.github/workflows/release-cli.yml";
const MANIFEST_SCHEMA: &str = "reason-release-manifest-v1";
const MIN_SPLIT_VERSION: &str = "0.5.0";
const MIN_GH_VERSION: &str = "2.93.0";
const MAX_MANIFEST_BYTES: usize = 1024 * 1024;
const MAX_CHECKSUM_BYTES: usize = 1024 * 1024;
const MAX_ARCHIVE_BYTES: usize = 512 * 1024 * 1024;

#[derive(Debug, Args)]
pub(crate) struct UpdateArgs {
    /// Check for an update without downloading/replacing the CLI archive.
    #[arg(long)]
    pub(crate) check: bool,
    /// Select an explicit split Reason CLI version instead of the newest published reason-v* release.
    #[arg(long, value_name = "VERSION")]
    pub(crate) version: Option<String>,
    /// Explicitly select an older split Reason CLI version. Never inferred from --version.
    #[arg(long, value_name = "VERSION", conflicts_with = "version")]
    pub(crate) rollback: Option<String>,
    /// Explicitly allow an update that changes Harness Engine SemVer.
    #[arg(long)]
    pub(crate) allow_engine_change: bool,
    /// Apply without an interactive confirmation prompt.
    #[arg(long)]
    pub(crate) yes: bool,
    #[arg(long, value_enum, default_value_t)]
    pub(crate) format: OutputFormat,
}

#[derive(Debug, Args)]
pub(crate) struct UninstallArgs {
    /// Preview what would be removed without mutating anything.
    #[arg(long)]
    pub(crate) dry_run: bool,
    /// Uninstall without an interactive confirmation prompt.
    #[arg(long)]
    pub(crate) yes: bool,
    /// Remove Reason-managed config.json and project-trust.json. Explicit-path session files are retained.
    #[arg(long)]
    pub(crate) purge_data: bool,
    /// Remove supported provider credentials from the native OS credential store. Environment variables are untouched.
    #[arg(long)]
    pub(crate) purge_credentials: bool,
    #[arg(long, value_enum, default_value_t)]
    pub(crate) format: OutputFormat,
}

impl UpdateArgs {
    pub(crate) const fn format(&self) -> OutputFormat {
        self.format
    }
}
impl UninstallArgs {
    pub(crate) const fn format(&self) -> OutputFormat {
        self.format
    }
}

#[derive(Debug, Clone, Deserialize)]
struct GithubRelease {
    tag_name: String,
    draft: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleaseManifest {
    schema_version: String,
    repository: String,
    signer_workflow: String,
    tag: String,
    cli_version: String,
    engine_version: String,
    git_commit: String,
    artifacts: Vec<ReleaseArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleaseArtifact {
    name: String,
    sha256: String,
    size_bytes: u64,
}

#[derive(Debug, Clone)]
struct VerifiedManifest {
    manifest: ReleaseManifest,
    cli_version: Version,
    engine_version: Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum LifecycleDirection {
    Current,
    Update,
    Rollback,
}

#[derive(Debug, Serialize)]
struct UpdateOutput {
    operation: &'static str,
    current_cli_version: String,
    current_engine_version: String,
    target_cli_version: String,
    target_engine_version: String,
    target_tag: String,
    release_url: String,
    git_commit: String,
    direction: LifecycleDirection,
    engine_change: bool,
    mutation_requested: bool,
    applied: bool,
    replacement_scheduled: bool,
    executable: String,
    provenance: &'static str,
}

#[derive(Debug, Serialize)]
struct UninstallOutput {
    operation: &'static str,
    dry_run: bool,
    executable: String,
    binary_removed: bool,
    binary_removal_scheduled: bool,
    data_files: Vec<String>,
    data_removed: usize,
    credentials_requested: bool,
    credentials_removed: usize,
    environment_credentials_untouched: bool,
    explicit_session_files_untouched: bool,
}

#[derive(Debug, Clone, Copy)]
struct PlatformSpec {
    asset: &'static str,
    archive_suffix: &'static str,
    executable: &'static str,
}

struct TempDir(PathBuf);
impl TempDir {
    fn create(label: &str) -> Result<Self, CliError> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| CliError::new("lifecycle_io", format!("system clock error: {error}")))?
            .as_nanos();
        let path = env::temp_dir().join(format!("reason-{label}-{}-{stamp}", std::process::id()));
        fs::create_dir(&path).map_err(|error| {
            CliError::new("lifecycle_io", format!("{}: {error}", path.display()))
        })?;
        Ok(Self(path))
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(crate) async fn run_update(args: UpdateArgs) -> Result<(), CliError> {
    let rollback_requested = args.rollback.is_some();
    let requested = args.rollback.as_deref().or(args.version.as_deref());
    let target = resolve_target_manifest(requested).await?;
    let current_cli = current_cli_version()?;
    let current_engine = current_engine_version()?;
    let direction = compare_direction(&current_cli, &target.cli_version);

    if rollback_requested {
        if direction != LifecycleDirection::Rollback {
            return Err(CliError::new(
                "rollback_direction",
                format!(
                    "rollback target {} must be older than current CLI {}",
                    target.cli_version, current_cli
                ),
            ));
        }
    } else if direction == LifecycleDirection::Rollback {
        return Err(CliError::new(
            "rollback_required",
            format!(
                "target {} is older than current {}; use `reason update --rollback {}` for an explicit downgrade",
                target.cli_version, current_cli, target.cli_version
            ),
        ));
    }

    let engine_change = current_engine != target.engine_version;
    if !args.check && engine_change && !args.allow_engine_change {
        return Err(engine_change_error(&current_engine, &target.engine_version));
    }
    let executable = current_executable()?;
    let operation = if rollback_requested {
        "rollback"
    } else {
        "update"
    };
    if args.check || direction == LifecycleDirection::Current {
        return emit_update(
            operation,
            &current_cli,
            &current_engine,
            &target,
            direction,
            engine_change,
            false,
            false,
            false,
            &executable,
            args.format,
        );
    }
    let verb = if rollback_requested {
        "Rollback"
    } else {
        "Update"
    };
    confirm_mutation(
        args.yes,
        args.format,
        &format!(
            "{verb} Reason CLI {current_cli} -> {} (Harness Engine {current_engine} -> {})?",
            target.cli_version, target.engine_version
        ),
    )?;
    let scheduled = apply_verified_release(&target, &executable).await?;
    emit_update(
        operation,
        &current_cli,
        &current_engine,
        &target,
        direction,
        engine_change,
        true,
        !scheduled,
        scheduled,
        &executable,
        args.format,
    )
}

pub(crate) fn run_uninstall(args: UninstallArgs) -> Result<(), CliError> {
    let executable = current_executable()?;
    let data_files = managed_data_files();
    if !args.dry_run {
        confirm_mutation(
            args.yes,
            args.format,
            &format!("Uninstall Reason CLI at {}?", executable.display()),
        )?;
    }

    let mut credentials_removed = 0usize;
    if args.purge_credentials && !args.dry_run {
        for provider in [
            Provider::Mistral,
            Provider::Google,
            Provider::Groq,
            Provider::Nvidia,
        ] {
            if secure_credentials::delete_provider_credential(provider)
                .map_err(|error| CliError::new(error.failure_class(), error.message))?
            {
                credentials_removed += 1;
            }
        }
    }

    let mut data_removed = 0usize;
    if args.purge_data && !args.dry_run {
        for path in &data_files {
            if path.exists() {
                fs::remove_file(path).map_err(|error| {
                    CliError::new("uninstall_io", format!("{}: {error}", path.display()))
                })?;
                data_removed += 1;
            }
        }
        if let Some(parent) = data_files.first().and_then(|path| path.parent()) {
            let _ = fs::remove_dir(parent);
        }
    }

    let scheduled = if args.dry_run {
        false
    } else {
        remove_current_executable(&executable)?
    };

    let output = UninstallOutput {
        operation: "uninstall",
        dry_run: args.dry_run,
        executable: executable.display().to_string(),
        binary_removed: !args.dry_run && !scheduled,
        binary_removal_scheduled: scheduled,
        data_files: data_files
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        data_removed,
        credentials_requested: args.purge_credentials,
        credentials_removed,
        environment_credentials_untouched: true,
        explicit_session_files_untouched: true,
    };
    match args.format {
        OutputFormat::Json => print_product_json("uninstall", &output).map_err(CliError::from),
        OutputFormat::Human => {
            println!("Reason CLI executable: {}", output.executable);
            if output.dry_run {
                println!("Dry run: no files or credentials were removed.");
            } else if output.binary_removal_scheduled {
                println!("Binary removal is scheduled immediately after this process exits.");
            } else {
                println!("Binary removed.");
            }
            if args.purge_data {
                println!("Managed data removed: {} file(s).", output.data_removed);
            } else {
                println!("Managed config/trust data retained.");
            }
            if args.purge_credentials {
                println!(
                    "Native OS credentials removed: {}.",
                    output.credentials_removed
                );
            } else {
                println!(
                    "Native OS credentials retained. Use `reason auth logout` before uninstall if desired."
                );
            }
            println!(
                "Explicit-path session files and provider environment variables are never deleted by uninstall."
            );
            Ok(())
        }
    }
}

async fn resolve_target_manifest(version: Option<&str>) -> Result<VerifiedManifest, CliError> {
    let target = match version {
        Some(value) => normalize_split_version(value)?,
        None => discover_latest_split_release().await?,
    };
    require_gh_verifier()?;
    let tag = format!("reason-v{target}");
    let temp = TempDir::create("manifest")?;
    let path = temp.path().join("release-manifest.json");
    let url = format!("{REPOSITORY_URL}/releases/download/{tag}/release-manifest.json");
    download_to(&url, &path, MAX_MANIFEST_BYTES).await?;
    verify_attestation(&path, &tag)?;
    let manifest = read_and_validate_manifest(&path, &tag, &target)?;
    let cli_version = Version::parse(&manifest.cli_version).map_err(|error| {
        CliError::new(
            "release_manifest",
            format!("invalid CLI version in manifest: {error}"),
        )
    })?;
    let engine_version = Version::parse(&manifest.engine_version).map_err(|error| {
        CliError::new(
            "release_manifest",
            format!("invalid Engine version in manifest: {error}"),
        )
    })?;
    Ok(VerifiedManifest {
        manifest,
        cli_version,
        engine_version,
    })
}

async fn discover_latest_split_release() -> Result<Version, CliError> {
    let client = release_client()?;
    let response = client
        .get(API_RELEASES_URL)
        .send()
        .await
        .map_err(network_error)?;
    if !response.status().is_success() {
        return Err(CliError::new(
            "release_discovery",
            format!(
                "GitHub release discovery returned HTTP {}",
                response.status()
            ),
        ));
    }
    let releases: Vec<GithubRelease> = response.json().await.map_err(network_error)?;
    releases
        .into_iter()
        .filter(|release| !release.draft)
        .filter_map(|release| release.tag_name.strip_prefix("reason-v").map(str::to_owned))
        .filter_map(|value| Version::parse(&value).ok())
        .filter(|version| *version >= minimum_split_version())
        .max()
        .ok_or_else(|| {
            CliError::new(
                "release_discovery",
                "no published split Reason CLI release (reason-v*) was found",
            )
        })
}

fn normalize_split_version(value: &str) -> Result<Version, CliError> {
    let raw = value.strip_prefix("reason-v").unwrap_or(value);
    if value.starts_with('v') && !value.starts_with("reason-v") {
        return Err(CliError::new(
            "historical_release_boundary",
            "historical unified v* releases are pre-attestation and are not supported by self-update/rollback; use a reason-v* release",
        ));
    }
    let version = Version::parse(raw).map_err(|error| {
        CliError::new(
            "version",
            format!("invalid Reason CLI version {value:?}: {error}"),
        )
    })?;
    if version < minimum_split_version() {
        return Err(CliError::new(
            "historical_release_boundary",
            format!(
                "Reason CLI {version} predates the reason-v* provenance contract; supported lifecycle management starts at {MIN_SPLIT_VERSION}"
            ),
        ));
    }
    Ok(version)
}

fn minimum_split_version() -> Version {
    Version::parse(MIN_SPLIT_VERSION).expect("constant semver")
}
fn current_cli_version() -> Result<Version, CliError> {
    Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|error| CliError::new("version", error.to_string()))
}
fn current_engine_version() -> Result<Version, CliError> {
    Version::parse(reasoning_harness_core::ENGINE_VERSION)
        .map_err(|error| CliError::new("version", error.to_string()))
}
fn compare_direction(current: &Version, target: &Version) -> LifecycleDirection {
    use std::cmp::Ordering;
    match target.cmp(current) {
        Ordering::Less => LifecycleDirection::Rollback,
        Ordering::Equal => LifecycleDirection::Current,
        Ordering::Greater => LifecycleDirection::Update,
    }
}

fn release_client() -> Result<reqwest::Client, CliError> {
    reqwest::Client::builder()
        .user_agent(format!("reason/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(network_error)
}

async fn download_to(url: &str, path: &Path, limit: usize) -> Result<(), CliError> {
    let mut response = release_client()?
        .get(url)
        .send()
        .await
        .map_err(network_error)?;
    if !response.status().is_success() {
        return Err(CliError::new(
            "release_download",
            format!("{url}: HTTP {}", response.status()),
        ));
    }
    if response
        .content_length()
        .is_some_and(|size| size > limit as u64)
    {
        return Err(CliError::new(
            "release_download",
            format!("{url}: payload exceeds {limit} byte limit"),
        ));
    }
    let mut file = fs::File::create(path)
        .map_err(|error| CliError::new("lifecycle_io", format!("{}: {error}", path.display())))?;
    let mut total = 0usize;
    while let Some(chunk) = response.chunk().await.map_err(network_error)? {
        total = total.saturating_add(chunk.len());
        if total > limit {
            drop(file);
            let _ = fs::remove_file(path);
            return Err(CliError::new(
                "release_download",
                format!("{url}: payload exceeds {limit} byte limit"),
            ));
        }
        file.write_all(&chunk).map_err(|error| {
            CliError::new("lifecycle_io", format!("{}: {error}", path.display()))
        })?;
    }
    file.sync_all()
        .map_err(|error| CliError::new("lifecycle_io", format!("{}: {error}", path.display())))
}

fn network_error(error: reqwest::Error) -> CliError {
    CliError::new(
        "release_network",
        format!("release network request failed: {error}"),
    )
}

fn require_gh_verifier() -> Result<(), CliError> {
    let output = Command::new("gh")
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .map_err(|error| {
            CliError::new(
                "provenance_verifier",
                format!("GitHub CLI >= {MIN_GH_VERSION} is required: {error}"),
            )
        })?;
    if !output.status.success() {
        return Err(CliError::new(
            "provenance_verifier",
            "could not execute `gh --version`",
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version_text = stdout
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("gh version "))
        .and_then(|rest| rest.split_whitespace().next())
        .ok_or_else(|| {
            CliError::new("provenance_verifier", "could not parse GitHub CLI version")
        })?;
    let version = Version::parse(version_text).map_err(|error| {
        CliError::new(
            "provenance_verifier",
            format!("invalid GitHub CLI version {version_text:?}: {error}"),
        )
    })?;
    let minimum = Version::parse(MIN_GH_VERSION).expect("constant semver");
    if version < minimum {
        return Err(CliError::new(
            "provenance_verifier",
            format!(
                "GitHub CLI {version} is too old; trusted release verification requires >= {minimum}"
            ),
        ));
    }
    Ok(())
}

fn verify_attestation(path: &Path, tag: &str) -> Result<(), CliError> {
    let source_ref = format!("refs/tags/{tag}");
    let output = Command::new("gh")
        .args(["attestation", "verify"])
        .arg(path)
        .args(["--repo", REPOSITORY])
        .args(["--signer-workflow", SIGNER_WORKFLOW])
        .args(["--source-ref", &source_ref])
        .arg("--deny-self-hosted-runners")
        .env("GH_HOST", "github.com")
        .stdin(Stdio::null())
        .output()
        .map_err(|error| {
            CliError::new(
                "provenance_verification",
                format!("could not execute gh attestation verify: {error}"),
            )
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CliError::new(
            "provenance_verification",
            format!(
                "release provenance verification failed for {}: {}",
                path.file_name()
                    .and_then(|v| v.to_str())
                    .unwrap_or("artifact"),
                stderr.trim()
            ),
        ));
    }
    Ok(())
}

fn read_and_validate_manifest(
    path: &Path,
    expected_tag: &str,
    expected_version: &Version,
) -> Result<ReleaseManifest, CliError> {
    let bytes = fs::read(path).map_err(|error| {
        CliError::new("release_manifest", format!("{}: {error}", path.display()))
    })?;
    let manifest: ReleaseManifest = serde_json::from_slice(&bytes).map_err(|error| {
        CliError::new(
            "release_manifest",
            format!("{}: invalid release manifest: {error}", path.display()),
        )
    })?;
    if manifest.schema_version != MANIFEST_SCHEMA
        || manifest.repository != REPOSITORY
        || manifest.signer_workflow != SIGNER_WORKFLOW
        || manifest.tag != expected_tag
        || manifest.cli_version != expected_version.to_string()
    {
        return Err(CliError::new(
            "release_manifest",
            "release manifest identity does not match the requested repository/workflow/tag/version",
        ));
    }
    if manifest.git_commit.len() != 40
        || !manifest.git_commit.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(CliError::new(
            "release_manifest",
            "release manifest git_commit must be a 40-character hexadecimal commit identity",
        ));
    }
    let engine = Version::parse(&manifest.engine_version).map_err(|error| {
        CliError::new(
            "release_manifest",
            format!("invalid engine_version: {error}"),
        )
    })?;
    let _ = engine;
    let mut names = std::collections::BTreeSet::new();
    for artifact in &manifest.artifacts {
        if !names.insert(&artifact.name) {
            return Err(CliError::new(
                "release_manifest",
                format!("duplicate artifact entry {}", artifact.name),
            ));
        }
        if artifact.size_bytes == 0
            || artifact.sha256.len() != 64
            || !artifact.sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(CliError::new(
                "release_manifest",
                format!("invalid artifact metadata for {}", artifact.name),
            ));
        }
    }
    Ok(manifest)
}

async fn apply_verified_release(
    target: &VerifiedManifest,
    executable: &Path,
) -> Result<bool, CliError> {
    let platform = platform_spec()?;
    let tag = &target.manifest.tag;
    let archive_name = format!(
        "reason-v{}-{}.{}",
        target.cli_version, platform.asset, platform.archive_suffix
    );
    let temp = TempDir::create("update-download")?;
    let archive = temp.path().join(&archive_name);
    let checksums = temp.path().join("SHA256SUMS");
    let base = format!("{REPOSITORY_URL}/releases/download/{tag}");
    download_to(
        &format!("{base}/{archive_name}"),
        &archive,
        MAX_ARCHIVE_BYTES,
    )
    .await?;
    download_to(
        &format!("{base}/SHA256SUMS"),
        &checksums,
        MAX_CHECKSUM_BYTES,
    )
    .await?;
    verify_attestation(&archive, tag)?;
    validate_and_replace_downloaded_release(target, executable, &archive, &checksums, platform)
}

fn validate_and_replace_downloaded_release(
    target: &VerifiedManifest,
    executable: &Path,
    archive: &Path,
    checksums: &Path,
    platform: PlatformSpec,
) -> Result<bool, CliError> {
    let archive_name = archive
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            CliError::new(
                "release_integrity",
                "release archive has no UTF-8 file name",
            )
        })?;
    let archive_entry = manifest_artifact(&target.manifest, archive_name)?;
    let checksum_entry = manifest_artifact(&target.manifest, "SHA256SUMS")?;
    verify_file_metadata(archive, archive_entry)?;
    verify_file_metadata(checksums, checksum_entry)?;
    verify_checksum_listing(checksums, archive_name, &archive_entry.sha256)?;

    let extract_temp = TempDir::create("update-extract")?;
    let extracted = extract_archive(archive, extract_temp.path(), target, platform)?;
    let version_output = Command::new(&extracted)
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .map_err(|error| {
            CliError::new(
                "release_binary",
                format!("could not execute downloaded Reason CLI: {error}"),
            )
        })?;
    let reported = String::from_utf8_lossy(&version_output.stdout)
        .trim()
        .to_string();
    let expected = format!("reason {}", target.cli_version);
    if !version_output.status.success() || reported != expected {
        return Err(CliError::new(
            "release_binary",
            format!("downloaded binary reported {reported:?}; expected {expected:?}"),
        ));
    }
    replace_current_executable(&extracted, executable)
}

fn manifest_artifact<'a>(
    manifest: &'a ReleaseManifest,
    name: &str,
) -> Result<&'a ReleaseArtifact, CliError> {
    manifest
        .artifacts
        .iter()
        .find(|entry| entry.name == name)
        .ok_or_else(|| {
            CliError::new(
                "release_manifest",
                format!("release manifest does not contain {name}"),
            )
        })
}

fn verify_file_metadata(path: &Path, expected: &ReleaseArtifact) -> Result<(), CliError> {
    let metadata = fs::metadata(path)
        .map_err(|error| CliError::new("lifecycle_io", format!("{}: {error}", path.display())))?;
    if metadata.len() != expected.size_bytes {
        return Err(CliError::new(
            "release_integrity",
            format!("{} size mismatch", expected.name),
        ));
    }
    let actual = sha256_file(path)?;
    if !actual.eq_ignore_ascii_case(&expected.sha256) {
        return Err(CliError::new(
            "release_integrity",
            format!(
                "{} SHA-256 does not match attested release manifest",
                expected.name
            ),
        ));
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, CliError> {
    let mut file = fs::File::open(path)
        .map_err(|error| CliError::new("lifecycle_io", format!("{}: {error}", path.display())))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|error| {
            CliError::new("lifecycle_io", format!("{}: {error}", path.display()))
        })?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let digest = hasher.finalize();
    let mut output = String::with_capacity(digest.len() * 2);
    use std::fmt::Write as _;
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("write to String cannot fail");
    }
    Ok(output)
}

fn verify_checksum_listing(
    path: &Path,
    archive_name: &str,
    expected: &str,
) -> Result<(), CliError> {
    let text = fs::read_to_string(path).map_err(|error| {
        CliError::new("release_integrity", format!("{}: {error}", path.display()))
    })?;
    let matches = text
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let digest = fields.next()?;
            let name = fields.next()?.trim_start_matches('*');
            (name == archive_name).then_some(digest)
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 || !matches[0].eq_ignore_ascii_case(expected) {
        return Err(CliError::new(
            "release_integrity",
            format!("SHA256SUMS does not bind {archive_name} to the attested manifest digest"),
        ));
    }
    Ok(())
}

fn platform_spec() -> Result<PlatformSpec, CliError> {
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => Ok(PlatformSpec {
            asset: "linux-x86_64",
            archive_suffix: "tar.gz",
            executable: "reason",
        }),
        ("macos", "aarch64") => Ok(PlatformSpec {
            asset: "macos-aarch64",
            archive_suffix: "tar.gz",
            executable: "reason",
        }),
        ("macos", "x86_64") => Ok(PlatformSpec {
            asset: "macos-x86_64",
            archive_suffix: "tar.gz",
            executable: "reason",
        }),
        ("windows", "x86_64") => Ok(PlatformSpec {
            asset: "windows-x86_64",
            archive_suffix: "zip",
            executable: "reason.exe",
        }),
        (os, arch) => Err(CliError::new(
            "unsupported_platform",
            format!("Reason lifecycle management does not support {os}/{arch}"),
        )),
    }
}

fn extract_archive(
    archive: &Path,
    temp: &Path,
    target: &VerifiedManifest,
    platform: PlatformSpec,
) -> Result<PathBuf, CliError> {
    let extract = temp.join("extract");
    fs::create_dir(&extract).map_err(|error| {
        CliError::new("lifecycle_io", format!("{}: {error}", extract.display()))
    })?;
    if env::consts::OS == "windows" {
        let shell = if Command::new("pwsh")
            .arg("-NoProfile")
            .arg("-Command")
            .arg("exit 0")
            .output()
            .is_ok()
        {
            "pwsh"
        } else {
            "powershell.exe"
        };
        let status = Command::new(shell)
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Expand-Archive -LiteralPath $args[0] -DestinationPath $args[1]",
            ])
            .arg(archive)
            .arg(&extract)
            .stdin(Stdio::null())
            .status()
            .map_err(|error| {
                CliError::new(
                    "release_archive",
                    format!("could not start PowerShell archive extraction: {error}"),
                )
            })?;
        if !status.success() {
            return Err(CliError::new(
                "release_archive",
                "PowerShell failed to extract release archive",
            ));
        }
    } else {
        let listing = Command::new("tar")
            .arg("-tzf")
            .arg(archive)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| {
                CliError::new(
                    "release_archive",
                    format!("could not inspect tar archive: {error}"),
                )
            })?;
        if !listing.status.success() {
            return Err(CliError::new(
                "release_archive",
                "release archive is not a valid tar.gz",
            ));
        }
        let listing = String::from_utf8_lossy(&listing.stdout);
        if listing.lines().any(|entry| {
            let path = Path::new(entry);
            path.is_absolute()
                || path
                    .components()
                    .any(|component| matches!(component, std::path::Component::ParentDir))
        }) {
            return Err(CliError::new(
                "release_archive",
                "release archive contains an unsafe path",
            ));
        }
        let status = Command::new("tar")
            .arg("-xzf")
            .arg(archive)
            .arg("-C")
            .arg(&extract)
            .stdin(Stdio::null())
            .status()
            .map_err(|error| {
                CliError::new("release_archive", format!("could not start tar: {error}"))
            })?;
        if !status.success() {
            return Err(CliError::new(
                "release_archive",
                "tar failed to extract release archive",
            ));
        }
    }
    let binary = extract
        .join(format!("reason-v{}-{}", target.cli_version, platform.asset))
        .join(platform.executable);
    let metadata = fs::symlink_metadata(&binary).map_err(|error| {
        CliError::new("release_archive", format!("{}: {error}", binary.display()))
    })?;
    if !metadata.file_type().is_file() {
        return Err(CliError::new(
            "release_archive",
            "release archive binary is missing or not a regular file",
        ));
    }
    Ok(binary)
}

fn current_executable() -> Result<PathBuf, CliError> {
    env::current_exe().map_err(|error| {
        CliError::new(
            "lifecycle_io",
            format!("cannot resolve current Reason executable: {error}"),
        )
    })
}

#[cfg(unix)]
fn replace_current_executable(source: &Path, destination: &Path) -> Result<bool, CliError> {
    use std::os::unix::fs::PermissionsExt;
    let parent = destination
        .parent()
        .ok_or_else(|| CliError::new("update_io", "current executable has no parent directory"))?;
    let staged = parent.join(format!(".reason.update.{}.tmp", std::process::id()));
    fs::copy(source, &staged)
        .map_err(|error| CliError::new("update_io", format!("{}: {error}", staged.display())))?;
    fs::set_permissions(&staged, fs::Permissions::from_mode(0o755))
        .map_err(|error| CliError::new("update_io", format!("{}: {error}", staged.display())))?;
    fs::rename(&staged, destination).map_err(|error| {
        let _ = fs::remove_file(&staged);
        CliError::new(
            "update_io",
            format!("replace {}: {error}", destination.display()),
        )
    })?;
    Ok(false)
}

#[cfg(windows)]
fn replace_current_executable(source: &Path, destination: &Path) -> Result<bool, CliError> {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    let parent = destination
        .parent()
        .ok_or_else(|| CliError::new("update_io", "current executable has no parent directory"))?;
    let staged = parent.join(format!(".reason.update.{}.exe", std::process::id()));
    fs::copy(source, &staged)
        .map_err(|error| CliError::new("update_io", format!("{}: {error}", staged.display())))?;
    let helper = parent.join(format!(".reason.update.{}.ps1", std::process::id()));
    let script = powershell_replace_script(std::process::id(), &staged, destination, false);
    fs::write(&helper, script)
        .map_err(|error| CliError::new("update_io", format!("{}: {error}", helper.display())))?;
    Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(&helper)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .spawn()
        .map_err(|error| {
            CliError::new(
                "update_io",
                format!("schedule Windows executable replacement: {error}"),
            )
        })?;
    Ok(true)
}

#[cfg(windows)]
fn powershell_replace_script(
    pid: u32,
    staged: &Path,
    destination: &Path,
    delete_only: bool,
) -> String {
    fn quote(path: &Path) -> String {
        format!("'{}'", path.display().to_string().replace('\'', "''"))
    }
    let mut script = format!(
        "$ErrorActionPreference='Stop'\nwhile (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ Start-Sleep -Milliseconds 100 }}\n"
    );
    if delete_only {
        script.push_str(&format!(
            "Remove-Item -LiteralPath {} -Force\n",
            quote(destination)
        ));
    } else {
        script.push_str(&format!(
            "Move-Item -LiteralPath {} -Destination {} -Force\n",
            quote(staged),
            quote(destination)
        ));
    }
    script
        .push_str("Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue\n");
    script
}

#[cfg(unix)]
fn remove_current_executable(path: &Path) -> Result<bool, CliError> {
    fs::remove_file(path)
        .map_err(|error| CliError::new("uninstall_io", format!("{}: {error}", path.display())))?;
    Ok(false)
}

#[cfg(windows)]
fn remove_current_executable(path: &Path) -> Result<bool, CliError> {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    let parent = path.parent().ok_or_else(|| {
        CliError::new("uninstall_io", "current executable has no parent directory")
    })?;
    let helper = parent.join(format!(".reason.uninstall.{}.ps1", std::process::id()));
    let script = powershell_replace_script(std::process::id(), Path::new(""), path, true);
    fs::write(&helper, script)
        .map_err(|error| CliError::new("uninstall_io", format!("{}: {error}", helper.display())))?;
    Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(&helper)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .spawn()
        .map_err(|error| {
            CliError::new(
                "uninstall_io",
                format!("schedule Windows uninstall: {error}"),
            )
        })?;
    Ok(true)
}

fn managed_data_files() -> Vec<PathBuf> {
    let Some(config) = user_config_path() else {
        return vec![];
    };
    let mut files = vec![config.clone()];
    if let Some(parent) = config.parent() {
        files.push(parent.join("project-trust.json"));
    }
    files
}

fn confirm_mutation(yes: bool, format: OutputFormat, prompt: &str) -> Result<(), CliError> {
    if yes {
        return Ok(());
    }
    if format == OutputFormat::Json || !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        return Err(CliError::new(
            "confirmation_required",
            format!("{prompt} Re-run with --yes to confirm."),
        ));
    }
    eprint!("{prompt} [y/N] ");
    io::stderr()
        .flush()
        .map_err(|error| CliError::new("confirmation", error.to_string()))?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|error| CliError::new("confirmation", error.to_string()))?;
    if matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
        Ok(())
    } else {
        Err(CliError::new("cancelled", "lifecycle mutation cancelled"))
    }
}

fn engine_change_error(current: &Version, target: &Version) -> CliError {
    CliError::new(
        "engine_change_confirmation_required",
        format!(
            "target release changes Harness Engine {current} -> {target}; inspect the change and re-run with --allow-engine-change to acknowledge the semantic/runtime identity change"
        ),
    )
}

#[allow(clippy::too_many_arguments)]
fn emit_update(
    operation: &'static str,
    current_cli: &Version,
    current_engine: &Version,
    target: &VerifiedManifest,
    direction: LifecycleDirection,
    engine_change: bool,
    mutation_requested: bool,
    applied: bool,
    replacement_scheduled: bool,
    executable: &Path,
    format: OutputFormat,
) -> Result<(), CliError> {
    let output = UpdateOutput {
        operation,
        current_cli_version: current_cli.to_string(),
        current_engine_version: current_engine.to_string(),
        target_cli_version: target.cli_version.to_string(),
        target_engine_version: target.engine_version.to_string(),
        target_tag: target.manifest.tag.clone(),
        release_url: format!("{REPOSITORY_URL}/releases/tag/{}", target.manifest.tag),
        git_commit: target.manifest.git_commit.clone(),
        direction,
        engine_change,
        mutation_requested,
        applied,
        replacement_scheduled,
        executable: executable.display().to_string(),
        provenance: "github_oidc_sigstore+manifest_sha256+release_sha256",
    };
    match format {
        OutputFormat::Json => print_product_json("update", &output).map_err(CliError::from),
        OutputFormat::Human => {
            println!(
                "Reason CLI: {} -> {} ({:?})",
                output.current_cli_version, output.target_cli_version, direction
            );
            println!(
                "Harness Engine: {} -> {}{}",
                output.current_engine_version,
                output.target_engine_version,
                if engine_change { " [CHANGE]" } else { "" }
            );
            println!("Release: {} @ {}", output.target_tag, output.git_commit);
            println!("Release notes: {}", output.release_url);
            if replacement_scheduled {
                println!(
                    "Verified replacement scheduled for process exit: {}",
                    output.executable
                );
            } else if applied {
                println!("Verified replacement applied: {}", output.executable);
            } else if direction == LifecycleDirection::Current {
                println!("Already at the selected Reason CLI version.");
            } else {
                println!("Check only: no executable was changed.");
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(cli: &str, engine: &str) -> ReleaseManifest {
        ReleaseManifest {
            schema_version: MANIFEST_SCHEMA.into(),
            repository: REPOSITORY.into(),
            signer_workflow: SIGNER_WORKFLOW.into(),
            tag: format!("reason-v{cli}"),
            cli_version: cli.into(),
            engine_version: engine.into(),
            git_commit: "a".repeat(40),
            artifacts: vec![
                ReleaseArtifact {
                    name: "SHA256SUMS".into(),
                    sha256: "b".repeat(64),
                    size_bytes: 10,
                },
                ReleaseArtifact {
                    name: format!("reason-v{cli}-linux-x86_64.tar.gz"),
                    sha256: "c".repeat(64),
                    size_bytes: 20,
                },
            ],
        }
    }

    #[test]
    fn split_version_boundary_and_direction_are_explicit() {
        assert_eq!(
            normalize_split_version("reason-v0.5.0").unwrap(),
            Version::parse("0.5.0").unwrap()
        );
        assert_eq!(
            normalize_split_version("0.6.0-beta.1").unwrap(),
            Version::parse("0.6.0-beta.1").unwrap()
        );
        assert_eq!(
            normalize_split_version("v0.4.2").unwrap_err().failure_class,
            "historical_release_boundary"
        );
        assert_eq!(
            normalize_split_version("0.4.2").unwrap_err().failure_class,
            "historical_release_boundary"
        );
        let current = Version::parse("0.5.1").unwrap();
        assert_eq!(
            compare_direction(&current, &Version::parse("0.5.2").unwrap()),
            LifecycleDirection::Update
        );
        assert_eq!(
            compare_direction(&current, &Version::parse("0.5.1").unwrap()),
            LifecycleDirection::Current
        );
        assert_eq!(
            compare_direction(&current, &Version::parse("0.5.0").unwrap()),
            LifecycleDirection::Rollback
        );
    }

    #[test]
    fn manifest_identity_and_checksum_listing_fail_closed() {
        let temp = TempDir::create("manifest-test").unwrap();
        let path = temp.path().join("release-manifest.json");
        fs::write(
            &path,
            serde_json::to_vec(&manifest("0.5.0", "0.4.2")).unwrap(),
        )
        .unwrap();
        let parsed =
            read_and_validate_manifest(&path, "reason-v0.5.0", &Version::parse("0.5.0").unwrap())
                .unwrap();
        assert_eq!(parsed.engine_version, "0.4.2");

        let sums = temp.path().join("SHA256SUMS");
        fs::write(
            &sums,
            format!("{}  reason-v0.5.0-linux-x86_64.tar.gz\n", "c".repeat(64)),
        )
        .unwrap();
        verify_checksum_listing(&sums, "reason-v0.5.0-linux-x86_64.tar.gz", &"c".repeat(64))
            .unwrap();
        fs::write(
            &sums,
            format!("{}  reason-v0.5.0-linux-x86_64.tar.gz\n", "d".repeat(64)),
        )
        .unwrap();
        assert_eq!(
            verify_checksum_listing(&sums, "reason-v0.5.0-linux-x86_64.tar.gz", &"c".repeat(64))
                .unwrap_err()
                .failure_class,
            "release_integrity"
        );
    }

    #[cfg(unix)]
    #[test]
    fn verified_downloaded_release_replaces_destination_after_integrity_and_version_checks() {
        use std::os::unix::fs::PermissionsExt;
        let temp = TempDir::create("downloaded-release-test").unwrap();
        let platform = platform_spec().unwrap();
        let cli = Version::parse("0.5.0").unwrap();
        let package_name = format!("reason-v{cli}-{}", platform.asset);
        let package = temp.path().join(&package_name);
        fs::create_dir(&package).unwrap();
        let packaged_binary = package.join("reason");
        fs::write(&packaged_binary, b"#!/bin/sh\necho 'reason 0.5.0'\n").unwrap();
        fs::set_permissions(&packaged_binary, fs::Permissions::from_mode(0o755)).unwrap();
        let archive_name = format!("{package_name}.tar.gz");
        let archive = temp.path().join(&archive_name);
        let status = Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(temp.path())
            .arg(&package_name)
            .status()
            .unwrap();
        assert!(status.success());
        let archive_sha = sha256_file(&archive).unwrap();
        let checksums = temp.path().join("SHA256SUMS");
        fs::write(&checksums, format!("{archive_sha}  {archive_name}\n")).unwrap();
        let checksum_sha = sha256_file(&checksums).unwrap();
        let target = VerifiedManifest {
            manifest: ReleaseManifest {
                schema_version: MANIFEST_SCHEMA.into(),
                repository: REPOSITORY.into(),
                signer_workflow: SIGNER_WORKFLOW.into(),
                tag: "reason-v0.5.0".into(),
                cli_version: "0.5.0".into(),
                engine_version: "0.4.2".into(),
                git_commit: "a".repeat(40),
                artifacts: vec![
                    ReleaseArtifact {
                        name: archive_name,
                        sha256: archive_sha,
                        size_bytes: fs::metadata(&archive).unwrap().len(),
                    },
                    ReleaseArtifact {
                        name: "SHA256SUMS".into(),
                        sha256: checksum_sha,
                        size_bytes: fs::metadata(&checksums).unwrap().len(),
                    },
                ],
            },
            cli_version: cli,
            engine_version: Version::parse("0.4.2").unwrap(),
        };
        let destination = temp.path().join("installed-reason");
        fs::write(&destination, b"old").unwrap();
        assert!(
            !validate_and_replace_downloaded_release(
                &target,
                &destination,
                &archive,
                &checksums,
                platform,
            )
            .unwrap()
        );
        let installed = fs::read_to_string(&destination).unwrap();
        assert!(installed.contains("reason 0.5.0"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_replacement_helper_waits_and_quotes_literal_paths() {
        let staged = Path::new(r"C:\Temp\Reason O'Brien\reason.new.exe");
        let destination = Path::new(r"C:\Program Files\Reason O'Brien\reason.exe");
        let script = powershell_replace_script(42, staged, destination, false);
        assert!(script.contains("Get-Process -Id 42"));
        assert!(script.contains("Reason O''Brien"));
        assert!(script.contains("Move-Item -LiteralPath"));
        assert!(script.contains("$PSCommandPath"));
    }

    #[cfg(unix)]
    #[test]
    fn unix_replacement_is_atomic_at_destination_path() {
        use std::os::unix::fs::PermissionsExt;
        let temp = TempDir::create("replace-test").unwrap();
        let source = temp.path().join("new-reason");
        let destination = temp.path().join("reason");
        fs::write(&source, b"new-binary").unwrap();
        fs::write(&destination, b"old-binary").unwrap();
        fs::set_permissions(&source, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(!replace_current_executable(&source, &destination).unwrap());
        assert_eq!(fs::read(&destination).unwrap(), b"new-binary");
        assert_eq!(
            fs::metadata(&destination).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }

    #[test]
    fn managed_data_scope_never_discovers_arbitrary_session_files() {
        let files = managed_data_files();
        for path in files {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            assert!(matches!(name, "config.json" | "project-trust.json"));
        }
    }
}
