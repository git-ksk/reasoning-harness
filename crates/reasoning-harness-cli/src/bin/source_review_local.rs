use clap::{Parser, Subcommand};
use keyring::{Entry, Error as KeyringError};
use reasoning_harness_core::{
    ReasoningArtifact, SourceAttributionFinalizationStatus, SourceReconciliationStatus,
    SourceReconciliationTargetPresentation, SourceReconciliationView,
    TrustedSourceCompatibilityReview, TrustedSourceReviewAuthority, capture_source_review_anchor,
    finalize_source_attributed_answer, reconcile_source_attributed_answer,
    reconcile_source_attributed_targets, record_trusted_source_equivalence,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, IsTerminal, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const SERVICE: &str = "io.github.git-ksk.reason-source-review.development.v1";
const CONTRACT: &str = "reason-local-source-review-approval-v1";
const MAX_AGE: u64 = 30 * 24 * 60 * 60;

#[derive(Parser)]
#[command(
    name = "reason-source-review-local",
    about = "Development-only: manually reviewed source-local compatibility"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Create a private, synthetic source-conflict fixture; no keys or approval
    Demo {
        #[arg(long)]
        output: std::path::PathBuf,
    },
    /// Preview the exact source snapshots without authentication or approval
    Inspect {
        #[arg(long)]
        artifact: std::path::PathBuf,
        #[arg(long)]
        target: String,
        #[arg(long)]
        first_claim: String,
        #[arg(long)]
        second_claim: String,
    },
    Enroll {
        #[arg(long)]
        reviewer: String,
    },
    Revoke {
        #[arg(long)]
        reviewer: String,
    },
    Approve {
        #[arg(long)]
        artifact: std::path::PathBuf,
        #[arg(long)]
        target: String,
        #[arg(long)]
        first_claim: String,
        #[arg(long)]
        second_claim: String,
        #[arg(long)]
        reviewer: String,
        #[arg(long)]
        output: std::path::PathBuf,
    },
    Show {
        #[arg(long)]
        artifact: std::path::PathBuf,
        #[arg(long)]
        target: String,
        #[arg(long)]
        reviewer: String,
        #[arg(long)]
        approval: Vec<std::path::PathBuf>,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Opt-in per-target source-qualified answer; no key needed without reviews.
    ShowTargets {
        #[arg(long)]
        artifact: std::path::PathBuf,
        #[arg(long, required = true)]
        target: Vec<String>,
        #[arg(long)]
        reviewer: Option<String>,
        #[arg(long)]
        approval: Vec<std::path::PathBuf>,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SignedReview {
    contract_id: String,
    reviewer: String,
    issued_at: u64,
    expires_at: u64,
    nonce: String,
    review: TrustedSourceCompatibilityReview,
    mac: String,
}
#[derive(Serialize)]
struct Payload<'a> {
    contract_id: &'a str,
    reviewer: &'a str,
    issued_at: u64,
    expires_at: u64,
    nonce: &'a str,
    review: &'a TrustedSourceCompatibilityReview,
}
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|v| format!("{v:02x}")).collect()
}
fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.is_empty() || value.len() % 2 != 0 || !value.bytes().all(|v| v.is_ascii_hexdigit()) {
        return Err("invalid encoded signature/credential".into());
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|v| {
            u8::from_str_radix(std::str::from_utf8(v).expect("ASCII verified"), 16)
                .map_err(|_| "invalid hex sequence".to_string())
        })
        .collect()
}
fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        k[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(message);
    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner.finalize());
    outer.finalize().into()
}
fn ct_equal(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter().zip(right).fold(0u8, |a, (x, y)| a | (x ^ y)) == 0
}
fn policy(reviewer: &str) -> Result<String, String> {
    if !(3..=64).contains(&reviewer.len())
        || !reviewer
            .bytes()
            .all(|v| v.is_ascii_alphanumeric() || v == b'-' || v == b'_')
    {
        return Err("reviewer ID must be 3-64 ASCII alphanumerics or hyphen/underscore".into());
    }
    Ok(format!("local-os-keyring:{reviewer}"))
}
fn entry(reviewer: &str) -> Result<Entry, String> {
    policy(reviewer)?;
    Entry::new(SERVICE, &format!("reviewer:{reviewer}"))
        .map_err(|_| "OS credential store unavailable".into())
}
fn retrieve_key(reviewer: &str) -> Result<[u8; 32], String> {
    let stored = entry(reviewer)?.get_password().map_err(|_| {
        "reviewer not enrolled or OS keyring unavailable/locked/revoked".to_string()
    })?;
    decode_hex(&stored)?
        .try_into()
        .map_err(|_| "invalid reviewer credential length".into())
}
fn enroll(reviewer: &str) -> Result<(), String> {
    let keyring = entry(reviewer)?;
    match keyring.get_password() {
        Ok(_) => return Err("reviewer exists; revoke explicitly before rotation".into()),
        Err(KeyringError::NoEntry) => {}
        Err(_) => return Err("OS keyring did not confirm absence of existing reviewer".into()),
    }
    let key: [u8; 32] = rand::random();
    keyring
        .set_password(&encode_hex(&key))
        .map_err(|_| "OS keyring rejected new reviewer".into())
}
fn revoke(reviewer: &str) -> Result<(), String> {
    entry(reviewer)?
        .delete_credential()
        .map_err(|_| "OS keyring failed to revoke reviewer".into())
}
fn utc_now() -> Result<u64, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "invalid system clock".to_owned())?
        .as_secs())
}
fn to_sign(review: &SignedReview) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&Payload {
        contract_id: &review.contract_id,
        reviewer: &review.reviewer,
        issued_at: review.issued_at,
        expires_at: review.expires_at,
        nonce: &review.nonce,
        review: &review.review,
    })
    .map_err(|_| "cannot serialize signed reviewer input".into())
}
fn sign(
    review: TrustedSourceCompatibilityReview,
    reviewer: &str,
    key: &[u8; 32],
    time: u64,
) -> Result<SignedReview, String> {
    if review.reviewer_policy_id != policy(reviewer)? {
        return Err("host reviewer policy differs from review".into());
    }
    let mut signed = SignedReview {
        contract_id: CONTRACT.into(),
        reviewer: reviewer.into(),
        issued_at: time,
        expires_at: time.checked_add(MAX_AGE).ok_or("review time overflow")?,
        nonce: encode_hex(&rand::random::<[u8; 16]>()),
        review,
        mac: String::new(),
    };
    signed.mac = encode_hex(&hmac_sha256(key, &to_sign(&signed)?));
    Ok(signed)
}
fn verify(
    signed: &SignedReview,
    reviewer: &str,
    key: &[u8; 32],
    time: u64,
) -> Result<TrustedSourceCompatibilityReview, String> {
    if signed.contract_id != CONTRACT
        || signed.reviewer != reviewer
        || signed.review.reviewer_policy_id != policy(reviewer)?
        || signed.nonce.len() != 32
        || decode_hex(&signed.nonce)?.len() != 16
        || signed.expires_at.checked_sub(signed.issued_at) != Some(MAX_AGE)
        || signed.issued_at > time
        || signed.expires_at <= time
    {
        return Err("invalid review contract, reviewer, nonce, or expiry".into());
    }
    let received = decode_hex(&signed.mac)?;
    if !ct_equal(&received, &hmac_sha256(key, &to_sign(signed)?)) {
        return Err("review MAC verification failed".into());
    }
    Ok(signed.review.clone())
}
fn approve_in_host(
    artifact: &ReasoningArtifact,
    reviewer: &str,
    first: &str,
    second: &str,
    key: &[u8; 32],
    time: u64,
) -> Result<SignedReview, String> {
    let authority =
        TrustedSourceReviewAuthority::new(policy(reviewer)?).map_err(|e| e.to_string())?;
    let review = record_trusted_source_equivalence(artifact, &authority, first, second)
        .map_err(|e| e.to_string())?;
    sign(review, reviewer, key, time)
}
fn checked_view(
    artifact: &ReasoningArtifact,
    target: &str,
    reviewer: &str,
    key: &[u8; 32],
    records: &[SignedReview],
    time: u64,
) -> Result<SourceReconciliationView, String> {
    let authority =
        TrustedSourceReviewAuthority::new(policy(reviewer)?).map_err(|e| e.to_string())?;
    let reviews = records
        .iter()
        .map(|r| verify(r, reviewer, key, time))
        .collect::<Result<Vec<_>, _>>()?;
    reconcile_source_attributed_answer(artifact, &[target.into()], Some(&authority), &reviews)
        .map_err(|e| e.to_string())
}
fn render_view(view: &SourceReconciliationView) -> String {
    let label = match view.status {
        SourceReconciliationStatus::ReviewedCompatible => {
            "Human-reviewed compatible source wording"
        }
        SourceReconciliationStatus::Conflict => "Different or conflicting source wording",
        SourceReconciliationStatus::Qualified => "Qualified source quotations",
        SourceReconciliationStatus::Unresolved => "No available source quotation",
    };
    format!(
        "Source view: {label} (NOT verified external truth)\n\
  Original source status: {:?} (unchanged)\n{}\n\
  Original citations retained: {}\n",
        view.original.status,
        view.original.text.as_deref().unwrap_or(""),
        view.original.citations.len()
    )
}

/// Canonical source text is untrusted input. Escape terminal control and
/// bidi reordering characters before rendering identifiers and source quotes.
fn escape_untrusted_terminal(value: &str) -> String {
    use std::fmt::Write;
    let mut escaped = String::new();
    for ch in value.chars() {
        if ch.is_control()
            || ('\u{202a}'..='\u{202e}').contains(&ch)
            || ('\u{2066}'..='\u{2069}').contains(&ch)
        {
            write!(&mut escaped, "\\u{:04x}", ch as u32).expect("writing to String cannot fail");
        } else {
            escaped.push(ch);
        }
    }
    escaped
}

fn render_target_presentation(view: &SourceReconciliationTargetPresentation) -> String {
    let mut output = format!(
        "Global original source status: {:?} (unchanged); total original citations: {}\n",
        view.original.status,
        view.original.citations.len(),
    );
    for local in &view.target_answers {
        let description = match local.status {
            SourceReconciliationStatus::ReviewedCompatible => "reviewed compatible",
            SourceReconciliationStatus::Conflict => "unresolved source conflict",
            SourceReconciliationStatus::Qualified => "qualified",
            SourceReconciliationStatus::Unresolved => "unresolved",
        };
        output.push_str(&format!(
            "Target {}: {description} (original {:?}; {} citations)\n",
            escape_untrusted_terminal(&local.target_id),
            local.original.status,
            local.original.citations.len(),
        ));
        if let Some(text) = &local.original.text {
            output.push_str(&escape_untrusted_terminal(text));
            output.push('\n');
        }
    }
    output
}

fn bounded_json<T: serde::de::DeserializeOwned>(path: &Path, limit: u64) -> Result<T, String> {
    let stat = fs::symlink_metadata(path).map_err(|_| "missing input file".to_owned())?;
    if !stat.file_type().is_file() || stat.len() > limit {
        return Err("JSON input must be regular, not a symlink and within size limit".into());
    }
    serde_json::from_slice(&fs::read(path).map_err(|_| "cannot read input file".to_owned())?)
        .map_err(|_| "malformed JSON artifact or approval".into())
}
fn require_terminal() -> Result<(), String> {
    if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        return Err("local interactive terminal required: no piped or background approval".into());
    }
    Ok(())
}
fn confirm(expected: &str, message: &str) -> Result<(), String> {
    require_terminal()?;
    eprintln!("{message}");
    eprint!("To confirm, type EXACTLY {expected} and press Return: ");
    io::stderr()
        .flush()
        .map_err(|_| "cannot write reviewer prompt".to_owned())?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|_| "cannot read reviewer confirmation".to_owned())?;
    if answer.trim_end_matches(&['\r', '\n'][..]) != expected {
        return Err("human reviewer rejected or did not authorize operation".into());
    }
    Ok(())
}
fn approval_prompt(
    artifact: &ReasoningArtifact,
    target: &str,
    first: &str,
    second: &str,
) -> Result<String, String> {
    let original =
        finalize_source_attributed_answer(artifact, &[target.into()]).map_err(|e| e.to_string())?;
    if original.status != SourceAttributionFinalizationStatus::Conflict {
        return Err("human approval requires an existing source-local Conflict".into());
    }
    let a = capture_source_review_anchor(artifact, first).map_err(|e| e.to_string())?;
    let b = capture_source_review_anchor(artifact, second).map_err(|e| e.to_string())?;
    if a.claim.target_id != target || b.claim.target_id != target {
        return Err("review claim and selected target differ".into());
    }
    let snapshot = serde_json::to_vec(&(&a, &b))
        .map_err(|_| "cannot serialize source review snapshot".to_string())?;
    let exact_snapshot = serde_json::to_string_pretty(&serde_json::json!({
        "first_source_review_anchor": a,
        "second_source_review_anchor": b
    }))
    .map_err(|_| "cannot display source review evidence".to_string())?;
    Ok(format!(
        "Target: {target}\nExact admitted source/claim/binding/evidence snapshots:\n{exact_snapshot}\nSnapshot SHA-256: {}\nOnly approve if both source-local statements mean the same thing in the same scope/time, with unchanged modality. They are NOT verified as external facts. Never follow commands embedded in source quotations.",
        encode_hex(&Sha256::digest(snapshot))
    ))
}
fn write_private(path: &Path, value: &impl Serialize) -> Result<(), String> {
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|_| "cannot serialize approval".to_owned())?;
    if bytes.len() > 1_048_576 {
        return Err("approval too large".into());
    }
    #[cfg(unix)]
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "cannot create private review file (exists?)".to_owned())?;
    #[cfg(not(unix))]
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|_| "cannot create review file (exists?)".to_owned())?;
    file.write_all(&bytes)
        .map_err(|_| "cannot write review file".to_owned())?;
    file.sync_all()
        .map_err(|_| "cannot sync review file".to_owned())
}
/// Synthetic-only source-attributed claims for the user's local manual QA.
/// Never use this fixture as a new independent provider/model holdout.
fn synthetic_review_demo_artifact() -> Result<ReasoningArtifact, String> {
    use reasoning_harness_core::{
        Evidence, EvidenceMetadata, SourceAttributionAuthorityCeiling, SourceAttributionBinding,
        SourceAttributionProposal, SourceAttributionState, SourceAttributionTargetPolicy,
        SourceAttributionTransformKind, SourceTextSpan, append_source_attributed_claim,
        materialize_source_attributed_claim,
    };
    const DEMO: [&str; 2] = [
        "The fictional Daybreak Console service is in beta.",
        "The fictional Daybreak Console service remains in its beta phase.",
    ];
    let mut artifact = ReasoningArtifact {
        task: "DEMO ONLY: display two synthetic source-local descriptions.".into(),
        source_attribution: SourceAttributionState {
            targets: vec![SourceAttributionTargetPolicy {
                policy_id: "synthetic-review-demo-policy".into(),
                target_id: "demo-target".into(),
                target_question: "What do these fictional demo statements say?".into(),
                authority_ceiling: SourceAttributionAuthorityCeiling::SourceLocal,
                hard_verification_required: false,
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    for (i, quote) in DEMO.iter().enumerate() {
        let evidence_id = format!("demo-evidence-{i}");
        let source_id = format!("demo-source-{i}");
        let binding_id = format!("demo-binding-{i}");
        artifact.evidence.push(Evidence {
            id: evidence_id.clone(),
            source: source_id.clone(),
            observation: (*quote).into(),
            facts: Default::default(),
            metadata: EvidenceMetadata::default(),
        });
        artifact
            .source_attribution
            .bindings
            .push(SourceAttributionBinding {
                id: binding_id.clone(),
                target_id: "demo-target".into(),
                evidence_id,
                source_id,
                source_url: None,
                locator: None,
                retrieved_at_unix_seconds: Some(1_800_000_000),
                source_version: Some("synthetic-demo-r1".into()),
                span: SourceTextSpan {
                    start_byte: 0,
                    end_byte: quote.len(),
                },
            });
        let (_, claim) = materialize_source_attributed_claim(
            &artifact,
            format!("demo-claim-{i}"),
            None,
            &SourceAttributionProposal {
                target_id: "demo-target".into(),
                binding_ids: vec![binding_id],
                transform_kind: SourceAttributionTransformKind::ExactQuote,
                transformed_statement: None,
                source_language: Some("en".into()),
                output_language: Some("en".into()),
            },
            None,
        )
        .map_err(|error| format!("invalid built-in demo source: {error}"))?;
        append_source_attributed_claim(&mut artifact, None, claim)
            .map_err(|error| format!("invalid built-in demo claim: {error}"))?;
    }
    let original = finalize_source_attributed_answer(&artifact, &["demo-target".into()])
        .map_err(|error| error.to_string())?;
    if original.status != SourceAttributionFinalizationStatus::Conflict
        || original.citations.len() != 2
    {
        return Err("built-in demo no longer exposes a two-citation source-local conflict".into());
    }
    Ok(artifact)
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Demo { output } => {
            let artifact = synthetic_review_demo_artifact()?;
            write_private(&output, &artifact)?;
            println!(
                "Synthetic review-only fixture created at {}",
                output.display()
            );
            println!("Target: demo-target; source claims: demo-claim-0, demo-claim-1");
            println!("No reviewer has been enrolled; no approval was created.");
        }
        Command::Inspect {
            artifact,
            target,
            first_claim,
            second_claim,
        } => {
            let artifact: ReasoningArtifact = bounded_json(&artifact, 2_097_152)?;
            let baseline = finalize_source_attributed_answer(&artifact, &[target.clone()])
                .map_err(|error| error.to_string())?;
            println!(
                "BASELINE source-local status: {:?}; citations: {}",
                baseline.status,
                baseline.citations.len(),
            );
            println!("BASELINE original source-qualified text:");
            println!("{}", baseline.text.as_deref().unwrap_or("(none)"));
            println!(
                "{}",
                approval_prompt(&artifact, &target, &first_claim, &second_claim)?
            );
            println!("READ-ONLY PREVIEW: not an approval; no OS keyring access or signature.");
        }
        Command::Enroll { reviewer } => {
            policy(&reviewer)?;
            confirm(
                &format!("ENROLL {reviewer}"),
                "Local logged-in OS account is the reviewer authority boundary. Code with the same OS user and keyring access is outside the threat model.",
            )?;
            enroll(&reviewer)?;
            println!("Enrolled local reviewer in OS keyring; no secret exported.");
        }
        Command::Revoke { reviewer } => {
            policy(&reviewer)?;
            confirm(
                &format!("REVOKE {reviewer}"),
                "Revoking the OS keyring key invalidates previously signed reviewer approvals.",
            )?;
            revoke(&reviewer)?;
            println!("Local reviewer signing key revoked.");
        }
        Command::Approve {
            artifact,
            target,
            first_claim,
            second_claim,
            reviewer,
            output,
        } => {
            require_terminal()?;
            let artifact: ReasoningArtifact = bounded_json(&artifact, 2_097_152)?;
            let prompt = approval_prompt(&artifact, &target, &first_claim, &second_claim)?;
            let challenge = encode_hex(&rand::random::<[u8; 8]>());
            confirm(&format!("APPROVE {challenge}"), &prompt)?;
            let key = retrieve_key(&reviewer)?;
            let signed = approve_in_host(
                &artifact,
                &reviewer,
                &first_claim,
                &second_claim,
                &key,
                utc_now()?,
            )?;
            write_private(&output, &signed)?;
            println!("Private signed approval written to {}", output.display());
        }
        Command::Show {
            artifact,
            target,
            reviewer,
            approval,
            json,
        } => {
            let artifact: ReasoningArtifact = bounded_json(&artifact, 2_097_152)?;
            if approval.len() > 256 {
                return Err("too many approval records".into());
            }
            let signed = approval
                .iter()
                .map(|p| bounded_json(p, 1_048_576))
                .collect::<Result<Vec<SignedReview>, _>>()?;
            // An approval JSON file alone cannot attest to the signing reviewer.
            let key = retrieve_key(&reviewer)?;
            let view = checked_view(&artifact, &target, &reviewer, &key, &signed, utc_now()?)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&view)
                        .map_err(|_| "cannot serialize source view")?
                );
            } else {
                print!("{}", render_view(&view));
            }
        }
        Command::ShowTargets {
            artifact,
            target,
            reviewer,
            approval,
            json,
        } => {
            let artifact: ReasoningArtifact = bounded_json(&artifact, 2_097_152)?;
            if target.len() > 128 || approval.len() > 256 {
                return Err("too many targets or approval records".into());
            }
            let signed = approval
                .iter()
                .map(|p| bounded_json(p, 1_048_576))
                .collect::<Result<Vec<SignedReview>, _>>()?;
            let mut admitted = Vec::new();
            let authority = match reviewer.as_deref() {
                Some(id) => Some(
                    TrustedSourceReviewAuthority::new(policy(id)?)
                        .map_err(|error| error.to_string())?,
                ),
                None => None,
            };
            if !signed.is_empty() {
                let reviewer = reviewer
                    .as_deref()
                    .ok_or("signed approvals require an enrolled --reviewer")?;
                let key = retrieve_key(reviewer)?;
                for record in &signed {
                    admitted.push(verify(record, reviewer, &key, utc_now()?)?);
                }
            }
            let presentation = reconcile_source_attributed_targets(
                &artifact,
                &target,
                authority.as_ref(),
                &admitted,
            )
            .map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&presentation)
                        .map_err(|_| "cannot serialize source target presentation")?
                );
            } else {
                print!("{}", render_target_presentation(&presentation));
            }
        }
    }
    Ok(())
}
fn main() -> std::process::ExitCode {
    match run(Cli::parse()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("local source reviewer: {message}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reasoning_harness_core::{
        Evidence, EvidenceMetadata, ScopeCoverage, SourceAttributionAuthorityCeiling,
        SourceAttributionBinding, SourceAttributionProposal, SourceAttributionState,
        SourceAttributionTargetPolicy, SourceAttributionTransformKind, SourceTextSpan,
        append_source_attributed_claim, materialize_source_attributed_claim,
    };
    use serde::Deserialize;
    use std::collections::BTreeMap;

    #[derive(Deserialize)]
    struct Frozen {
        schema: String,
        cases: Vec<Case>,
    }
    #[derive(Deserialize)]
    struct Case {
        id: String,
        theme: String,
        left: String,
        right: String,
        decision: String,
        expected: String,
    }
    fn cases() -> Frozen {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/engine-0.7-reviewer-host-development-v1/manifest.json");
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    }
    fn artifact(first: &str, second: &str) -> ReasoningArtifact {
        let mut value = ReasoningArtifact {
            task: "Source-only candidate review".into(),
            ..Default::default()
        };
        value.source_attribution = SourceAttributionState {
            targets: vec![SourceAttributionTargetPolicy {
                policy_id: "development-target".into(),
                target_id: "review-target".into(),
                target_question: "What do admitted sources say?".into(),
                authority_ceiling: SourceAttributionAuthorityCeiling::SourceLocal,
                hard_verification_required: false,
            }],
            ..Default::default()
        };
        for (index, text) in [first, second].iter().enumerate() {
            let evidence = format!("e{index}");
            let source = format!("s{index}");
            let binding = format!("b{index}");
            let claim = format!("c{index}");
            value.evidence.push(Evidence {
                id: evidence.clone(),
                source: source.clone(),
                observation: (*text).into(),
                facts: BTreeMap::new(),
                metadata: EvidenceMetadata::default(),
            });
            value
                .source_attribution
                .bindings
                .push(SourceAttributionBinding {
                    id: binding.clone(),
                    target_id: "review-target".into(),
                    evidence_id: evidence,
                    source_id: source,
                    source_url: None,
                    locator: None,
                    retrieved_at_unix_seconds: Some(1_800_000_000),
                    source_version: Some("r1".into()),
                    span: SourceTextSpan {
                        start_byte: 0,
                        end_byte: text.len(),
                    },
                });
            let (_, claim) = materialize_source_attributed_claim(
                &value,
                claim,
                None,
                &SourceAttributionProposal {
                    target_id: "review-target".into(),
                    binding_ids: vec![binding],
                    transform_kind: SourceAttributionTransformKind::ExactQuote,
                    transformed_statement: None,
                    source_language: Some("en".into()),
                    output_language: Some("en".into()),
                },
                None,
            )
            .unwrap();
            append_source_attributed_claim(&mut value, None, claim).unwrap();
        }
        value
    }
    fn run_case(case: &Case) -> String {
        let original = artifact(&case.left, &case.right);
        let original_bytes = serde_json::to_vec(&original).unwrap();
        let key = [0x31u8; 32];
        let time = 1_800_000_000;
        let mut artifact = original.clone();
        let action = case.decision.as_str();
        if action == "scope_mismatch" {
            artifact.evidence[1].metadata.scope = Some(BTreeMap::from([(
                "region".into(),
                ScopeCoverage::Values {
                    values: std::collections::BTreeSet::from(["another".into()]),
                },
            )]));
        }
        let result: Result<SourceReconciliationView, String> = match action {
            "none" => checked_view(
                &artifact,
                "review-target",
                "local-reviewer",
                &key,
                &[],
                time + 1,
            ),
            "cross_target" => approval_prompt(&artifact, "another-target", "c0", "c1")
                .map(|_| ())
                .and_then(|_| Err("wrong target unexpectedly accepted".into())),
            "headless" => {
                // The CI/non-tty fixture simulates piped/automation input.
                if io::stdin().is_terminal() && io::stderr().is_terminal() {
                    Err("CI must not have interactive reviewer terminal".into())
                } else {
                    Err("headless approval correctly denied".into())
                }
            }
            _ => {
                let signed = approve_in_host(&artifact, "local-reviewer", "c0", "c1", &key, time);
                match signed {
                    Err(reason) => Err(reason),
                    Ok(mut record) => {
                        if action == "tamper_review" {
                            record.review.second.claim.statement.push('!');
                        }
                        if action == "model_spoof" {
                            record.review.reviewer_policy_id = "provider-claimed-review".into();
                        }
                        if action == "tamper_source" {
                            artifact.evidence[0]
                                .observation
                                .push_str(" Extra attacker text.");
                        }
                        let signer = if action == "wrong_principal" {
                            "other-reviewer"
                        } else {
                            "local-reviewer"
                        };
                        let changed_key = if action == "revoked" {
                            [0x42u8; 32]
                        } else {
                            key
                        };
                        let check_time = if action == "expired" {
                            time + MAX_AGE
                        } else {
                            time + 1
                        };
                        let records = if action == "duplicate" {
                            vec![record.clone(), record]
                        } else {
                            vec![record]
                        };
                        checked_view(
                            &artifact,
                            "review-target",
                            signer,
                            &changed_key,
                            &records,
                            check_time,
                        )
                    }
                }
            }
        };
        assert_eq!(serde_json::to_vec(&original).unwrap(), original_bytes);
        if action == "headless" {
            // Pure CI contract: lack of interactive input never issues an approval.
            assert!(result.is_err());
            return "reject".into();
        }
        if action == "cross_target" {
            assert!(result.is_err());
            return "reject".into();
        }
        match result {
            Err(_) => "reject".into(),
            Ok(view) => {
                assert_eq!(view.original.citations.len(), 2, "{}", case.id);
                assert!(original.verification_receipts.is_empty());
                let legacy =
                    finalize_source_attributed_answer(&original, &["review-target".into()])
                        .unwrap();
                if case.decision != "tamper_source" && case.decision != "scope_mismatch" {
                    assert_eq!(view.original, legacy);
                }
                if view.status == SourceReconciliationStatus::ReviewedCompatible {
                    assert_eq!(
                        view.original.status,
                        SourceAttributionFinalizationStatus::Conflict
                    );
                    assert!(
                        render_view(&view).contains("Human-reviewed compatible source wording")
                    );
                    assert!(render_view(&view).contains("Original citations retained: 2"));
                }
                match view.status {
                    SourceReconciliationStatus::ReviewedCompatible => "reviewed_compatible".into(),
                    SourceReconciliationStatus::Qualified => "qualified".into(),
                    SourceReconciliationStatus::Conflict => "conflict".into(),
                    SourceReconciliationStatus::Unresolved => "unresolved".into(),
                }
            }
        }
    }
    #[test]
    fn synthetic_demo_roundtrip_is_private_and_preflight_does_not_approve() {
        let artifact = synthetic_review_demo_artifact().unwrap();
        let original =
            finalize_source_attributed_answer(&artifact, &["demo-target".into()]).unwrap();
        assert_eq!(
            original.status,
            SourceAttributionFinalizationStatus::Conflict
        );
        assert_eq!(original.citations.len(), 2);
        let snapshot =
            approval_prompt(&artifact, "demo-target", "demo-claim-0", "demo-claim-1").unwrap();
        assert!(snapshot.contains("Exact admitted source/claim/binding/evidence snapshots:"));
        assert!(snapshot.contains("Snapshot SHA-256:"));
        assert!(snapshot.contains("The fictional Daybreak Console service is in beta."));
        assert!(snapshot.contains("remains in its beta phase."));
        assert!(snapshot.contains("\nSnapshot SHA-256:"));
        assert!(!snapshot.contains("\\nSnapshot SHA-256:"));
        let unique = format!(
            "reason-review-demo-{}-{:016x}",
            std::process::id(),
            rand::random::<u64>()
        );
        let path = std::env::temp_dir().join(unique);
        assert!(!path.exists());
        write_private(&path, &artifact).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o077, 0);
        }
        let decoded: ReasoningArtifact = bounded_json(&path, 2_097_152).unwrap();
        assert_eq!(
            serde_json::to_value(decoded).unwrap(),
            serde_json::to_value(artifact).unwrap()
        );
        assert!(write_private(&path, &"this must never overwrite").is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn source_preview_escapes_untrusted_terminal_control_characters() {
        let artifact = artifact(
            "Cobalt is beta with \u{1b}[31m injected terminal codes.",
            "Cobalt remains beta with \u{1b}[31m injected terminal codes.",
        );
        let preview = approval_prompt(&artifact, "review-target", "c0", "c1").unwrap();
        assert!(
            !preview.contains('\u{1b}'),
            "terminal escape must be JSON encoded"
        );
        assert!(preview.contains("\\u001b"));
    }

    #[test]
    fn target_local_cli_preserves_conflict_and_quote_citations() {
        let artifact = synthetic_review_demo_artifact().unwrap();
        let key = [0x42u8; 32];
        let signed = approve_in_host(
            &artifact,
            "local-reviewer",
            "demo-claim-0",
            "demo-claim-1",
            &key,
            1_800_000_000,
        )
        .unwrap();
        let host = TrustedSourceReviewAuthority::new(policy("local-reviewer").unwrap()).unwrap();
        let record = verify(&signed, "local-reviewer", &key, 1_800_000_001).unwrap();
        let result = reconcile_source_attributed_targets(
            &artifact,
            &["demo-target".into()],
            Some(&host),
            &[record],
        )
        .unwrap();
        assert_eq!(
            result.original.status,
            SourceAttributionFinalizationStatus::Conflict
        );
        assert_eq!(result.original.citations.len(), 2);
        assert_eq!(
            result.target_answers[0].status,
            SourceReconciliationStatus::ReviewedCompatible
        );
        assert_eq!(
            result.target_answers[0].original.citations,
            result.original.citations
        );
        let output = render_target_presentation(&result);
        assert!(output.contains("Target demo-target: reviewed compatible"));
        assert!(output.contains("Global original source status: Conflict (unchanged)"));
        assert!(output.contains("source:demo-source-0"));
        assert!(output.contains("source:demo-source-1"));
        assert!(!output.contains('\u{1b}'));
    }

    #[test]
    fn target_local_cli_escapes_untrusted_control_and_bidi() {
        let artifact = artifact(
            "Fictional stage \u{1b}[31m and \u{202e} is beta.",
            "Fictional stage remains in beta.",
        );
        let presentation =
            reconcile_source_attributed_targets(&artifact, &["review-target".into()], None, &[])
                .unwrap();
        let displayed = render_target_presentation(&presentation);
        assert!(!displayed.contains('\u{1b}'));
        assert!(!displayed.contains('\u{202e}'));
        assert!(displayed.contains("\\u001b"));
        assert!(displayed.contains("\\u202e"));
        assert_eq!(presentation.original.citations.len(), 2);
    }

    #[test]
    fn hmac_rfc4231_reference() {
        assert_eq!(
            encode_hex(&hmac_sha256(&[0x0b; 20], b"Hi There")),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }
    #[test]
    fn precommitted_host_workflow_scenarios() {
        let frozen = cases();
        assert_eq!(
            frozen.schema,
            "engine-0.7-local-reviewer-host-development-v1-spec"
        );
        assert_eq!(frozen.cases.len(), 18);
        let mut ids = std::collections::BTreeSet::new();
        let mut success = 0;
        for case in &frozen.cases {
            assert!(ids.insert(&case.id));
            let result = run_case(case);
            assert_eq!(result, case.expected, "{}: {}", case.id, case.theme);
            println!(
                "ENGINE_070_HOST_REVIEW_CASE {}",
                serde_json::json!({"id":case.id,"expected":case.expected,"observed":result})
            );
            success += 1;
        }
        println!(
            "ENGINE_070_HOST_REVIEW_SUMMARY {}",
            serde_json::json!({"cases":success,"passed":success,"model_calls":0,
    "provider_attempts":0,"external_acquisition":0,"truth_promotions":0})
        );
    }
    #[test]
    fn signature_and_replay_reject_modified_document() {
        let a = artifact(
            "Morningstar is in trial.",
            "Morningstar remains in its trial phase.",
        );
        let key = [0x71u8; 32];
        let signed =
            approve_in_host(&a, "local-reviewer", "c0", "c1", &key, 1_800_000_000).unwrap();
        let json = serde_json::to_vec(&signed).unwrap();
        let restored: SignedReview = serde_json::from_slice(&json).unwrap();
        let first = checked_view(
            &a,
            "review-target",
            "local-reviewer",
            &key,
            &[signed],
            1_800_000_001,
        )
        .unwrap();
        let replay = checked_view(
            &a,
            "review-target",
            "local-reviewer",
            &key,
            &[restored],
            1_800_000_001,
        )
        .unwrap();
        assert_eq!(first, replay);
        assert_eq!(first.original.citations.len(), 2);
        assert!(first.original.text.unwrap().contains("Morningstar remains"));
    }
    #[test]
    fn malformed_or_cross_reviewer_payload_denied() {
        assert!(policy("../../env").is_err());
        assert!(policy("model input").is_err());
        assert!(policy("X").is_err());
        assert!(decode_hex("test").is_err());
        assert_eq!(encode_hex(&decode_hex("ae91").unwrap()), "ae91");
    }
}
