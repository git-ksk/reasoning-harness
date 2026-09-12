use super::*;
use fs2::FileExt;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};

static SESSION_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) const MANAGED_SESSION_CONTRACT_ID: &str = "reason-managed-session-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManagedContext {
    source: String,
    observation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManagedTurn {
    prompt: String,
    typed_session: SessionFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ManagedSession {
    schema_version: String,
    id: String,
    short_id: String,
    title: String,
    project_path: String,
    created_at_unix_ms: u64,
    updated_at_unix_ms: u64,
    #[serde(default)]
    memory_start_turn: usize,
    #[serde(default)]
    contexts: Vec<ManagedContext>,
    #[serde(default)]
    turns: Vec<ManagedTurn>,
    #[serde(skip)]
    source_digest: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ManagedSessionSummary {
    pub id: String,
    pub short_id: String,
    pub title: String,
    pub project_path: String,
    pub turns: usize,
    pub updated_at_unix_ms: u64,
}

#[derive(Debug, Serialize)]
pub(super) struct ManagedSessionListOutput {
    pub session_contract: &'static str,
    pub sessions: Vec<ManagedSessionSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub problems: Vec<ManagedSessionProblem>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ManagedSessionProblem {
    pub file_name: String,
    pub state: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ManagedSessionExportOutput {
    pub session_contract: &'static str,
    pub id: String,
    pub short_id: String,
    pub output_path: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ManagedSessionMutationOutput {
    pub session_contract: &'static str,
    pub operation: &'static str,
    pub dry_run: bool,
    pub selected: Vec<String>,
    pub removed: usize,
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
        .unwrap_or_default()
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("write to String cannot fail");
    }
    output
}

pub(super) fn managed_root_path() -> Result<PathBuf, CliError> {
    let config = user_config_path().ok_or_else(|| {
        CliError::new(
            "session_io",
            "cannot determine Reason user data directory; set REASON_HOME or a supported user home/config environment variable",
        )
    })?;
    let parent = config.parent().ok_or_else(|| {
        CliError::new(
            "session_io",
            format!("{}: invalid Reason user config path", config.display()),
        )
    })?;
    Ok(parent.join("sessions"))
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("sha256:{}", hex_lower(&Sha256::digest(bytes)))
}

fn store_lock(root: &Path) -> Result<fs::File, CliError> {
    if let Some(parent) = root.parent() {
        local_privacy::ensure_private_directory(parent)
            .map_err(|error| CliError::new("session_privacy", error))?;
    }
    local_privacy::ensure_private_directory(root)
        .map_err(|error| CliError::new("session_privacy", error))?;
    let lock_path = root.join(".store.lock");
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|error| {
            CliError::new("session_io", format!("{}: {error}", lock_path.display()))
        })?;
    local_privacy::ensure_private_file(&lock_path)
        .map_err(|error| CliError::new("session_privacy", error))?;
    file.try_lock_exclusive().map_err(|error| {
        CliError::new(
            "session_locked",
            format!("managed session store is busy: {error}"),
        )
    })?;
    Ok(file)
}

fn cleanup_interrupted_temps(root: &Path) -> Result<(), CliError> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).map_err(|error| {
        CliError::new(
            "session_io",
            format!("read managed session directory {}: {error}", root.display()),
        )
    })? {
        let entry = entry.map_err(|error| CliError::new("session_io", error.to_string()))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.contains(".tmp-") {
            let path = entry.path();
            fs::remove_file(&path).map_err(|error| {
                CliError::new(
                    "session_io",
                    format!("remove interrupted temp {}: {error}", path.display()),
                )
            })?;
        }
    }
    Ok(())
}

#[cfg(windows)]
fn atomic_replace(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    const MOVEFILE_REPLACE_EXISTING: u32 = 0x1;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x8;
    unsafe extern "system" {
        fn MoveFileExW(existing: *const u16, new_name: *const u16, flags: u32) -> i32;
    }
    let source = source
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let result = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn atomic_replace(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> io::Result<()> {
    fs::File::open(path)?.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn verify_expected_source(path: &Path, expected: Option<&str>) -> Result<(), CliError> {
    match (fs::read(path), expected) {
        (Ok(bytes), Some(expected)) if digest_bytes(&bytes) == expected => Ok(()),
        (Ok(_), None) => Err(CliError::new(
            "session_conflict",
            "managed session was created by another process before this save; reload before retrying",
        )),
        (Ok(_), Some(_)) => Err(CliError::new(
            "session_conflict",
            "managed session changed in another process; reload before retrying to avoid lost updates",
        )),
        (Err(error), Some(_)) if error.kind() == io::ErrorKind::NotFound => Err(CliError::new(
            "session_conflict",
            "managed session disappeared after it was loaded; refusing to recreate stale state",
        )),
        (Err(error), None) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        (Err(error), _) => Err(CliError::new(
            "session_io",
            format!("{}: {error}", path.display()),
        )),
    }
}

pub(super) fn current_project_key() -> Result<String, CliError> {
    let cwd = env::current_dir()
        .map_err(|error| CliError::new("session_context", format!("current directory: {error}")))?;
    let canonical = fs::canonicalize(&cwd).map_err(|error| {
        CliError::new(
            "session_context",
            format!(
                "{}: cannot resolve current project path: {error}",
                cwd.display()
            ),
        )
    })?;
    canonical.to_str().map(str::to_string).ok_or_else(|| {
        CliError::new(
            "session_context",
            format!(
                "{}: managed sessions require a UTF-8 canonical project path",
                canonical.display()
            ),
        )
    })
}

fn generated_identity(project_path: &str) -> (String, String, u64) {
    let now = now_unix_ms();
    let counter = SESSION_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let material = format!(
        "{project_path}\0{now}\0{nanos}\0{}\0{counter}",
        std::process::id()
    );
    let digest = Sha256::digest(material.as_bytes());
    let hex = hex_lower(&digest);
    let short_id = hex[..12].to_string();
    (format!("managed-{now}-{short_id}"), short_id, now)
}

impl ManagedSession {
    pub(super) fn new(project_path: String) -> Self {
        let (id, short_id, now) = generated_identity(&project_path);
        Self {
            schema_version: MANAGED_SESSION_CONTRACT_ID.into(),
            id,
            short_id,
            title: "New session".into(),
            project_path,
            created_at_unix_ms: now,
            updated_at_unix_ms: now,
            memory_start_turn: 0,
            contexts: vec![],
            turns: vec![],
            source_digest: None,
        }
    }

    pub(super) fn short_id(&self) -> &str {
        &self.short_id
    }
    pub(super) fn title(&self) -> &str {
        &self.title
    }
    pub(super) fn turns_len(&self) -> usize {
        self.turns.len()
    }

    pub(super) fn last_runtime(&self) -> Option<&SessionRuntimeIdentity> {
        self.turns.last().map(|turn| &turn.typed_session.runtime)
    }

    pub(super) fn conversation_context(&self) -> Vec<String> {
        let mut result = self
            .contexts
            .iter()
            .map(|context| {
                format!(
                    "Persisted untrusted context from {}:\n{}",
                    context.source, context.observation
                )
            })
            .collect::<Vec<_>>();
        for turn in self.turns.iter().skip(self.memory_start_turn) {
            let exposed = turn
                .typed_session
                .turns
                .last()
                .map(|record| exposed_finalization_text(&record.finalization))
                .unwrap_or_else(|| "No exposed answer text is available.".into());
            result.push(format!(
                "Prior interactive exchange (untrusted conversation memory; re-verify before relying on it):\nUser: {}\nReason: {}",
                turn.prompt, exposed
            ));
        }
        result
    }

    pub(super) fn add_context_file(&mut self, path: &Path) -> Result<(), CliError> {
        let metadata = fs::metadata(path)
            .map_err(|error| CliError::new("input", format!("{}: {error}", path.display())))?;
        if !metadata.is_file() {
            return Err(CliError::new(
                "input",
                format!("{}: context requires a regular file", path.display()),
            ));
        }
        if metadata.len() > MAX_CONTEXT_FILE_BYTES {
            return Err(CliError::new(
                "input",
                format!(
                    "{}: context file exceeds {} bytes",
                    path.display(),
                    MAX_CONTEXT_FILE_BYTES
                ),
            ));
        }
        let observation = fs::read_to_string(path)
            .map_err(|error| CliError::new("input", format!("{}: {error}", path.display())))?;
        let current_total = self
            .contexts
            .iter()
            .map(|context| context.observation.len())
            .sum::<usize>();
        if current_total.saturating_add(observation.len()) > MAX_CONTEXT_TOTAL_BYTES {
            return Err(CliError::new(
                "input",
                format!("managed session context exceeds {MAX_CONTEXT_TOTAL_BYTES} bytes"),
            ));
        }
        let source = path.display().to_string();
        if let Some(existing) = self
            .contexts
            .iter_mut()
            .find(|context| context.source == source)
        {
            existing.observation = observation;
        } else {
            self.contexts.push(ManagedContext {
                source,
                observation,
            });
        }
        self.updated_at_unix_ms = now_unix_ms();
        Ok(())
    }

    pub(super) fn context_sources(&self) -> impl Iterator<Item = &str> {
        self.contexts.iter().map(|context| context.source.as_str())
    }

    pub(super) fn clear_context(&mut self) {
        self.contexts.clear();
        self.memory_start_turn = self.turns.len();
        self.updated_at_unix_ms = now_unix_ms();
    }

    pub(super) fn append_turn(
        &mut self,
        output: &NaturalOutput,
        safety_profile: AnswerSafetyProfileArg,
    ) -> Result<(), CliError> {
        let turn_number = self.turns.len() + 1;
        let thread_id = format!("{}-turn-{turn_number}", self.id);
        let mut thread = ReasoningThread::new(thread_id.clone())
            .map_err(|error| CliError::new("session_state", error.to_string()))?;
        thread
            .record_task(
                session_event_id(1, "task", 0),
                format!("managed-task-{thread_id}"),
                output.task.clone(),
            )
            .map_err(|error| CliError::new("session_state", error.to_string()))?;
        let runtime = session_runtime_from_output(output, safety_profile)?;
        let mut typed_session = SessionFile {
            schema_version: SESSION_CONTRACT_ID.into(),
            runtime,
            thread,
            turns: vec![],
        };
        record_natural_turn(&mut typed_session, output, None)?;
        validate_session_runtime(&typed_session)?;

        if self.turns.is_empty() {
            self.title = title_from_prompt(&output.task);
        }
        self.turns.push(ManagedTurn {
            prompt: output.task.clone(),
            typed_session,
        });
        self.updated_at_unix_ms = now_unix_ms();
        Ok(())
    }

    fn summary(&self) -> ManagedSessionSummary {
        ManagedSessionSummary {
            id: self.id.clone(),
            short_id: self.short_id.clone(),
            title: self.title.clone(),
            project_path: self.project_path.clone(),
            turns: self.turns.len(),
            updated_at_unix_ms: self.updated_at_unix_ms,
        }
    }
}

fn title_from_prompt(prompt: &str) -> String {
    let collapsed = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = collapsed.chars();
    let title = chars.by_ref().take(60).collect::<String>();
    if chars.next().is_some() {
        format!("{title}…")
    } else if title.is_empty() {
        "Untitled session".into()
    } else {
        title
    }
}

fn exposed_finalization_text(finalization: &FinalizationResult) -> String {
    match finalization.status {
        FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer => {
            finalization
                .text
                .clone()
                .unwrap_or_else(|| "No exposed answer text is available.".into())
        }
        FinalizationStatus::Abstain => {
            "I cannot provide a grounded answer because verified state is contradictory.".into()
        }
        FinalizationStatus::Unresolved | FinalizationStatus::RequiresVerification => {
            "I cannot support a complete answer from the currently verified evidence.".into()
        }
    }
}

fn validate(session: &ManagedSession) -> Result<(), CliError> {
    if session.schema_version != MANAGED_SESSION_CONTRACT_ID {
        return Err(CliError::new(
            "session_incompatible",
            format!(
                "managed session contract mismatch: expected {MANAGED_SESSION_CONTRACT_ID}, got {}",
                session.schema_version
            ),
        ));
    }
    if session.id.trim().is_empty()
        || session.short_id.trim().is_empty()
        || session.project_path.trim().is_empty()
    {
        return Err(CliError::new(
            "session_invalid",
            "managed session identity/project path must not be empty",
        ));
    }
    if session.memory_start_turn > session.turns.len() {
        return Err(CliError::new(
            "session_invalid",
            "managed session memory boundary exceeds recorded turns",
        ));
    }
    for turn in &session.turns {
        if turn.prompt.trim().is_empty() {
            return Err(CliError::new(
                "session_invalid",
                "managed session contains an empty prompt",
            ));
        }
        validate_session_runtime(&turn.typed_session)?;
    }
    Ok(())
}

fn path_for(session: &ManagedSession) -> Result<PathBuf, CliError> {
    Ok(managed_root_path()?.join(format!("{}.json", session.id)))
}

pub(super) fn save(session: &mut ManagedSession) -> Result<(), CliError> {
    validate(session)?;
    let root = managed_root_path()?;
    let _lock = store_lock(&root)?;
    cleanup_interrupted_temps(&root)?;
    let path = path_for(session)?;
    verify_expected_source(&path, session.source_digest.as_deref())?;
    let bytes = serde_json::to_vec_pretty(session).map_err(|error| {
        CliError::new(
            "session_invalid",
            format!("serialize managed session: {error}"),
        )
    })?;
    let counter = SESSION_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temporary = path.with_extension(format!("json.tmp-{}-{counter}", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| {
            CliError::new(
                "session_io",
                format!(
                    "write managed session temp {}: {error}",
                    temporary.display()
                ),
            )
        })?;
    local_privacy::ensure_private_file(&temporary)
        .map_err(|error| CliError::new("session_privacy", error))?;
    file.write_all(&bytes).map_err(|error| {
        CliError::new(
            "session_io",
            format!(
                "write managed session temp {}: {error}",
                temporary.display()
            ),
        )
    })?;
    file.sync_all().map_err(|error| {
        CliError::new(
            "session_io",
            format!("sync managed session temp {}: {error}", temporary.display()),
        )
    })?;
    drop(file);
    if let Err(error) = atomic_replace(&temporary, &path) {
        let _ = fs::remove_file(&temporary);
        return Err(CliError::new(
            "session_io",
            format!(
                "atomically commit managed session {}: {error}",
                path.display()
            ),
        ));
    }
    local_privacy::ensure_private_file(&path)
        .map_err(|error| CliError::new("session_privacy", error))?;
    sync_directory(&root).map_err(|error| {
        CliError::new(
            "session_io",
            format!("sync managed session directory {}: {error}", root.display()),
        )
    })?;
    session.source_digest = Some(digest_bytes(&bytes));
    Ok(())
}

fn load_path(path: &Path) -> Result<ManagedSession, CliError> {
    local_privacy::ensure_private_file(path)
        .map_err(|error| CliError::new("session_privacy", error))?;
    let bytes = fs::read(path)
        .map_err(|error| CliError::new("session_io", format!("{}: {error}", path.display())))?;
    let mut session: ManagedSession = serde_json::from_slice(&bytes).map_err(|error| {
        CliError::new(
            "session_invalid",
            format!("invalid managed session {}: {error}", path.display()),
        )
    })?;
    validate(&session)?;
    session.source_digest = Some(digest_bytes(&bytes));
    Ok(session)
}

fn scan_all() -> Result<(Vec<ManagedSession>, Vec<ManagedSessionProblem>), CliError> {
    let root = managed_root_path()?;
    if !root.exists() {
        return Ok((vec![], vec![]));
    }
    let _lock = store_lock(&root)?;
    cleanup_interrupted_temps(&root)?;
    let mut sessions = Vec::new();
    let mut problems = Vec::new();
    for entry in fs::read_dir(&root).map_err(|error| {
        CliError::new(
            "session_io",
            format!("read managed session directory {}: {error}", root.display()),
        )
    })? {
        let entry = entry.map_err(|error| CliError::new("session_io", error.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        match load_path(&path) {
            Ok(session) => sessions.push(session),
            Err(error) => problems.push(ManagedSessionProblem {
                file_name: path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("<non-utf8-session>")
                    .to_string(),
                state: if error.failure_class == "session_incompatible" {
                    "incompatible"
                } else {
                    "corrupt"
                },
                message: error.message,
            }),
        }
    }
    sessions.sort_by_key(|session| std::cmp::Reverse(session.updated_at_unix_ms));
    problems.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    Ok((sessions, problems))
}

fn load_all() -> Result<Vec<ManagedSession>, CliError> {
    scan_all().map(|(sessions, _)| sessions)
}

pub(super) fn list() -> Result<ManagedSessionListOutput, CliError> {
    let (sessions, problems) = scan_all()?;
    Ok(ManagedSessionListOutput {
        session_contract: MANAGED_SESSION_CONTRACT_ID,
        sessions: sessions
            .into_iter()
            .map(|session| session.summary())
            .collect(),
        problems,
    })
}

pub(super) fn load_latest_for_project(project_path: &str) -> Result<ManagedSession, CliError> {
    load_all()?
        .into_iter()
        .find(|session| session.project_path == project_path)
        .ok_or_else(|| {
            CliError::new(
                "session_not_found",
                "no compatible managed session exists for this project; start `reason` or inspect `reason session list`",
            )
        })
}

pub(super) fn load_by_selector(selector: &str) -> Result<ManagedSession, CliError> {
    let matches = load_all()?
        .into_iter()
        .filter(|session| session.id == selector || session.short_id == selector)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Err(CliError::new(
            "session_not_found",
            format!("managed session {selector:?} was not found; inspect `reason session list`"),
        )),
        [session] => Ok(session.clone()),
        _ => Err(CliError::new(
            "session_ambiguous",
            format!("managed session selector {selector:?} is ambiguous; use the full id"),
        )),
    }
}

pub(super) fn export(
    selector: &str,
    output: &Path,
) -> Result<ManagedSessionExportOutput, CliError> {
    let session = load_by_selector(selector)?;
    let mut bytes = serde_json::to_vec_pretty(&session).map_err(|error| {
        CliError::new(
            "session_invalid",
            format!("serialize managed session export: {error}"),
        )
    })?;
    bytes.push(b'\n');
    local_privacy::write_new_private_file(output, &bytes)
        .map_err(|error| CliError::new("session_export", error))?;
    Ok(ManagedSessionExportOutput {
        session_contract: MANAGED_SESSION_CONTRACT_ID,
        id: session.id.clone(),
        short_id: session.short_id.clone(),
        output_path: output.display().to_string(),
    })
}

pub(super) fn delete(
    selector: &str,
    dry_run: bool,
) -> Result<ManagedSessionMutationOutput, CliError> {
    let session = load_by_selector(selector)?;
    let root = managed_root_path()?;
    let path = path_for(&session)?;
    let _lock = store_lock(&root)?;
    verify_expected_source(&path, session.source_digest.as_deref())?;
    if !dry_run {
        fs::remove_file(&path)
            .map_err(|error| CliError::new("session_io", format!("{}: {error}", path.display())))?;
        sync_directory(&root).map_err(|error| {
            CliError::new(
                "session_io",
                format!("sync managed session directory {}: {error}", root.display()),
            )
        })?;
    }
    Ok(ManagedSessionMutationOutput {
        session_contract: MANAGED_SESSION_CONTRACT_ID,
        operation: "delete",
        dry_run,
        selected: vec![session.short_id],
        removed: usize::from(!dry_run),
    })
}

pub(super) fn purge(
    all: bool,
    older_than_days: Option<u64>,
    dry_run: bool,
) -> Result<ManagedSessionMutationOutput, CliError> {
    if all == older_than_days.is_some() {
        return Err(CliError::new(
            "input",
            "session purge requires exactly one of --all or --older-than-days DAYS",
        ));
    }
    let (sessions, problems) = scan_all()?;
    let cutoff = older_than_days
        .map(|days| now_unix_ms().saturating_sub(days.saturating_mul(24 * 60 * 60 * 1000)));
    let selected_sessions = sessions
        .into_iter()
        .filter(|session| all || cutoff.is_some_and(|cutoff| session.updated_at_unix_ms <= cutoff))
        .collect::<Vec<_>>();
    let root = managed_root_path()?;
    let _lock = store_lock(&root)?;
    let mut selected = selected_sessions
        .iter()
        .map(|session| session.short_id.clone())
        .collect::<Vec<_>>();
    for session in &selected_sessions {
        let path = path_for(session)?;
        verify_expected_source(&path, session.source_digest.as_deref())?;
    }
    let mut problem_files = Vec::new();
    if all {
        for problem in problems {
            problem_files.push(problem.file_name.clone());
            selected.push(format!("{}:{}", problem.state, problem.file_name));
        }
    }
    let mut removed = 0usize;
    if !dry_run {
        for session in &selected_sessions {
            fs::remove_file(path_for(session)?).map_err(|error| {
                CliError::new(
                    "session_io",
                    format!("purge managed session {}: {error}", session.short_id),
                )
            })?;
            removed += 1;
        }
        for name in &problem_files {
            let path = root.join(name);
            if path.parent() == Some(root.as_path())
                && path.extension().and_then(|v| v.to_str()) == Some("json")
            {
                fs::remove_file(&path).map_err(|error| {
                    CliError::new(
                        "session_io",
                        format!("purge managed session problem {}: {error}", path.display()),
                    )
                })?;
                removed += 1;
            }
        }
        sync_directory(&root).map_err(|error| {
            CliError::new(
                "session_io",
                format!("sync managed session directory {}: {error}", root.display()),
            )
        })?;
    }
    Ok(ManagedSessionMutationOutput {
        session_contract: MANAGED_SESSION_CONTRACT_ID,
        operation: "purge",
        dry_run,
        selected,
        removed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_identity_is_stable_inside_record_and_title_is_bounded() {
        let session = ManagedSession::new("/tmp/project".into());
        assert!(session.id.contains(session.short_id()));
        assert_eq!(session.project_path, "/tmp/project");
        assert_eq!(
            title_from_prompt("  hello   from   reason  "),
            "hello from reason"
        );
        assert!(title_from_prompt(&"x".repeat(100)).chars().count() <= 61);
    }

    #[test]
    fn generated_managed_session_ids_do_not_collide_in_one_process() {
        let first = ManagedSession::new("/tmp/project".into());
        let second = ManagedSession::new("/tmp/project".into());
        assert_ne!(first.id, second.id);
        assert_ne!(first.short_id, second.short_id);
    }

    fn test_path(label: &str) -> PathBuf {
        let counter = SESSION_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        env::temp_dir().join(format!("reason-{label}-{}-{counter}", std::process::id()))
    }

    #[test]
    fn source_digest_is_not_part_of_v1_wire_format() {
        let mut session = ManagedSession::new("/tmp/project".into());
        session.source_digest = Some("sha256:test".into());
        let json = serde_json::to_string(&session).unwrap();
        assert!(!json.contains("source_digest"));
        let decoded: ManagedSession = serde_json::from_str(&json).unwrap();
        assert!(decoded.source_digest.is_none());
    }

    #[test]
    fn optimistic_source_check_rejects_stale_overwrite() {
        let path = test_path("digest-conflict");
        fs::write(&path, b"one").unwrap();
        let expected = digest_bytes(b"one");
        verify_expected_source(&path, Some(&expected)).unwrap();
        fs::write(&path, b"two").unwrap();
        let error = verify_expected_source(&path, Some(&expected)).unwrap_err();
        assert_eq!(error.failure_class, "session_conflict");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn atomic_replace_overwrites_existing_destination_without_remove_gap() {
        let root = test_path("atomic-root");
        fs::create_dir_all(&root).unwrap();
        let source = root.join("source.tmp");
        let destination = root.join("session.json");
        fs::write(&source, b"new").unwrap();
        fs::write(&destination, b"old").unwrap();
        atomic_replace(&source, &destination).unwrap();
        assert_eq!(fs::read(&destination).unwrap(), b"new");
        assert!(!source.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn advisory_store_lock_excludes_second_writer() {
        let private_parent = test_path("lock-parent");
        fs::create_dir_all(&private_parent).unwrap();
        let root = private_parent.join("sessions");
        let first = store_lock(&root).unwrap();
        let second_path = root.join(".store.lock");
        let second = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&second_path)
            .unwrap();
        assert!(second.try_lock_exclusive().is_err());
        drop(first);
        second.try_lock_exclusive().unwrap();
        FileExt::unlock(&second).unwrap();
        let _ = fs::remove_dir_all(private_parent);
    }

    #[test]
    fn managed_validation_rejects_memory_boundary_past_turns() {
        let mut session = ManagedSession::new("/tmp/project".into());
        session.memory_start_turn = 1;
        assert!(validate(&session).is_err());
    }
}
