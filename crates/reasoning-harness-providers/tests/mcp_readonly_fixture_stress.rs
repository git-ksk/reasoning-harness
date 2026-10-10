//! Unfrozen black-box stress test for the immutable legacy MCP v1 adapter.
//! The existing mcp_readonly.rs implementation and inline fixtures are untouched.
#![cfg(unix)]

use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use reasoning_harness_core::{
    Proposition, ResolutionAdapterErrorKind, ResolutionReason, ResolutionRequest,
    ResolutionRequestBudget, ResolutionResolver, ResolutionResolverContribution, ResolutionTarget,
    ResolverClass,
};
use reasoning_harness_providers::{McpReadOnlyResolver, McpReadOnlyResolverConfig};

static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(reply: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "reason-mcp-v1-stress-{nonce}-{}-{}.sh",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        let mut script = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o700)
            .open(&path)
            .unwrap();
        writeln!(script, "#!/bin/sh\nread request\nprintf '%s\\n' '{reply}'").unwrap();
        drop(script);
        Self(path)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_file(&self.0).ok();
    }
}

fn request() -> ResolutionRequest {
    ResolutionRequest {
        id: "resolution:service.region".into(),
        reason: ResolutionReason::MissingSupport,
        target: ResolutionTarget::Proposition {
            proposition: Proposition {
                key: "service.region".into(),
                value: "eu-west-1".into(),
            },
        },
        resolver_class: ResolverClass::EvidenceAcquisition,
        budget: ResolutionRequestBudget::default(),
    }
}

const OPAQUE: &str = r#"{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0","result":{"content":[{"type":"text","text":"probably eu-west-1"}]}}"#;
const TOOL_ERROR: &str = r#"{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0","result":{"content":[{"type":"text","text":"backend denied"}],"isError":true}}"#;

#[test]
fn parallel_legacy_mcp_fixtures_preserve_fail_closed_observations() {
    // Parallel script creation plus subprocess startup exercises collision,
    // stdin/stdout and cleanup under contention. Do not retry failed requests.
    let workers: Vec<_> = (0..8)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..4 {
                    let opaque = Fixture::new(OPAQUE);
                    let resolver =
                        McpReadOnlyResolver::new(McpReadOnlyResolverConfig::with_defaults(
                            "fixture-server",
                            opaque.0.clone(),
                            "lookup",
                            "mcp:fixture:lookup",
                        ));
                    let output = resolver
                        .resolve(&request(), 0)
                        .expect("opaque transport must succeed");
                    match output.contribution {
                        ResolutionResolverContribution::AcquiredEvidence { evidence } => {
                            assert_eq!(evidence.len(), 1);
                            assert!(
                                evidence[0].facts.is_empty(),
                                "opaque content must not promote facts"
                            );
                        }
                        other => panic!("expected acquired opaque evidence, got {other:?}"),
                    }
                    let tool_error = Fixture::new(TOOL_ERROR);
                    let resolver =
                        McpReadOnlyResolver::new(McpReadOnlyResolverConfig::with_defaults(
                            "fixture-server",
                            tool_error.0.clone(),
                            "lookup",
                            "mcp:fixture:lookup",
                        ));
                    let failure = resolver
                        .resolve(&request(), 0)
                        .expect_err("tool error must fail closed");
                    assert_eq!(failure.kind, ResolutionAdapterErrorKind::ToolExecution);
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().expect("fixture worker must not panic");
    }
}
