use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{BenchmarkCaseResult, BenchmarkComparison, aggregate_benchmark};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusSuite {
    Claim,
    Causal,
    Assumption,
    EvidenceQualification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusDifficulty {
    Basic,
    Standard,
    Stress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusScoringMode {
    DeterministicOracle,
    SoftJudge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusRedistribution {
    Redistributable,
    Restricted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusCaseStatus {
    Active,
    Deprecated,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpusCaseMetadata {
    pub case_id: String,
    pub fixture_id: String,
    pub suite: CorpusSuite,
    pub fixture_path: String,
    pub category: String,
    pub difficulty: CorpusDifficulty,
    pub difficulty_rationale: String,
    pub scoring_mode: CorpusScoringMode,
    pub provenance: String,
    pub redistribution: CorpusRedistribution,
    pub contamination_note: String,
    pub status: CorpusCaseStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpusManifest {
    pub manifest_schema_version: u32,
    pub corpus_version: String,
    pub score_compatibility_id: String,
    pub description: String,
    pub cases: Vec<CorpusCaseMetadata>,
}

impl CorpusManifest {
    pub fn score_compatible_with(&self, other: &Self) -> bool {
        self.score_compatibility_id == other.score_compatibility_id
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CorpusSliceComparison {
    pub label: String,
    pub cases: usize,
    pub comparison: BenchmarkComparison,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClaimCorpusSummary {
    pub corpus_version: String,
    pub score_compatibility_id: String,
    pub suite: CorpusSuite,
    pub cases: usize,
    pub by_category: Vec<CorpusSliceComparison>,
    pub by_difficulty: Vec<CorpusSliceComparison>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CorpusError {
    #[error("corpus manifest schema version must be at least 1")]
    InvalidSchemaVersion,
    #[error("corpus version must not be empty")]
    EmptyCorpusVersion,
    #[error("score compatibility id must not be empty")]
    EmptyCompatibilityId,
    #[error("corpus description must not be empty")]
    EmptyDescription,
    #[error("duplicate corpus case id: {case_id}")]
    DuplicateCaseId { case_id: String },
    #[error("duplicate corpus fixture path: {fixture_path}")]
    DuplicateFixturePath { fixture_path: String },
    #[error("duplicate active fixture id {fixture_id} in suite {suite:?}")]
    DuplicateSuiteFixtureId {
        suite: CorpusSuite,
        fixture_id: String,
    },
    #[error("invalid corpus metadata for case {case_id}: {field} must not be empty")]
    EmptyCaseField {
        case_id: String,
        field: &'static str,
    },
    #[error("active corpus case {case_id} must not declare superseded_by")]
    ActiveCaseSuperseded { case_id: String },
    #[error("superseded corpus case {case_id} must declare superseded_by")]
    SupersededCaseMissingTarget { case_id: String },
    #[error("superseded corpus case {case_id} references missing target {target}")]
    SupersededTargetMissing { case_id: String, target: String },
    #[error("superseded corpus case {case_id} cannot supersede itself")]
    SupersededTargetSelf { case_id: String },
    #[error("duplicate claim benchmark result fixture id: {fixture_id}")]
    DuplicateClaimResult { fixture_id: String },
    #[error("claim benchmark result has no active corpus metadata: {fixture_id}")]
    ClaimResultMissingMetadata { fixture_id: String },
    #[error("active claim corpus case was not evaluated: {fixture_id}")]
    ClaimCorpusCaseMissingResult { fixture_id: String },
}

pub fn validate_corpus_manifest(manifest: &CorpusManifest) -> Result<(), CorpusError> {
    if manifest.manifest_schema_version == 0 {
        return Err(CorpusError::InvalidSchemaVersion);
    }
    if manifest.corpus_version.trim().is_empty() {
        return Err(CorpusError::EmptyCorpusVersion);
    }
    if manifest.score_compatibility_id.trim().is_empty() {
        return Err(CorpusError::EmptyCompatibilityId);
    }
    if manifest.description.trim().is_empty() {
        return Err(CorpusError::EmptyDescription);
    }

    let all_case_ids = manifest
        .cases
        .iter()
        .map(|case| case.case_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut case_ids = BTreeSet::new();
    let mut fixture_paths = BTreeSet::new();
    let mut active_suite_fixture_ids = BTreeSet::new();
    for case in &manifest.cases {
        if !case_ids.insert(case.case_id.as_str()) {
            return Err(CorpusError::DuplicateCaseId {
                case_id: case.case_id.clone(),
            });
        }
        if !fixture_paths.insert(case.fixture_path.as_str()) {
            return Err(CorpusError::DuplicateFixturePath {
                fixture_path: case.fixture_path.clone(),
            });
        }
        for (field, value) in [
            ("case_id", case.case_id.as_str()),
            ("fixture_id", case.fixture_id.as_str()),
            ("fixture_path", case.fixture_path.as_str()),
            ("category", case.category.as_str()),
            ("difficulty_rationale", case.difficulty_rationale.as_str()),
            ("provenance", case.provenance.as_str()),
            ("contamination_note", case.contamination_note.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(CorpusError::EmptyCaseField {
                    case_id: case.case_id.clone(),
                    field,
                });
            }
        }

        match case.status {
            CorpusCaseStatus::Active => {
                if case.superseded_by.is_some() {
                    return Err(CorpusError::ActiveCaseSuperseded {
                        case_id: case.case_id.clone(),
                    });
                }
                if !active_suite_fixture_ids.insert((case.suite, case.fixture_id.as_str())) {
                    return Err(CorpusError::DuplicateSuiteFixtureId {
                        suite: case.suite,
                        fixture_id: case.fixture_id.clone(),
                    });
                }
            }
            CorpusCaseStatus::Superseded => {
                let Some(target) = case.superseded_by.as_deref() else {
                    return Err(CorpusError::SupersededCaseMissingTarget {
                        case_id: case.case_id.clone(),
                    });
                };
                if target == case.case_id {
                    return Err(CorpusError::SupersededTargetSelf {
                        case_id: case.case_id.clone(),
                    });
                }
                if !all_case_ids.contains(target) {
                    return Err(CorpusError::SupersededTargetMissing {
                        case_id: case.case_id.clone(),
                        target: target.into(),
                    });
                }
            }
            CorpusCaseStatus::Deprecated => {}
        }
    }
    Ok(())
}

pub fn aggregate_claim_corpus(
    manifest: &CorpusManifest,
    results: &[BenchmarkCaseResult],
) -> Result<ClaimCorpusSummary, CorpusError> {
    validate_corpus_manifest(manifest)?;

    let active_claim_cases = manifest
        .cases
        .iter()
        .filter(|case| case.suite == CorpusSuite::Claim && case.status == CorpusCaseStatus::Active)
        .collect::<Vec<_>>();
    let metadata_by_fixture = active_claim_cases
        .iter()
        .map(|case| (case.fixture_id.as_str(), *case))
        .collect::<BTreeMap<_, _>>();

    let mut result_ids = BTreeSet::new();
    for result in results {
        if !result_ids.insert(result.fixture_id.as_str()) {
            return Err(CorpusError::DuplicateClaimResult {
                fixture_id: result.fixture_id.clone(),
            });
        }
        if !metadata_by_fixture.contains_key(result.fixture_id.as_str()) {
            return Err(CorpusError::ClaimResultMissingMetadata {
                fixture_id: result.fixture_id.clone(),
            });
        }
    }
    for case in &active_claim_cases {
        if !result_ids.contains(case.fixture_id.as_str()) {
            return Err(CorpusError::ClaimCorpusCaseMissingResult {
                fixture_id: case.fixture_id.clone(),
            });
        }
    }

    let mut by_category = BTreeMap::<String, Vec<BenchmarkCaseResult>>::new();
    let mut by_difficulty = BTreeMap::<CorpusDifficulty, Vec<BenchmarkCaseResult>>::new();
    for result in results {
        let metadata = metadata_by_fixture[result.fixture_id.as_str()];
        by_category
            .entry(metadata.category.clone())
            .or_default()
            .push(result.clone());
        by_difficulty
            .entry(metadata.difficulty)
            .or_default()
            .push(result.clone());
    }

    Ok(ClaimCorpusSummary {
        corpus_version: manifest.corpus_version.clone(),
        score_compatibility_id: manifest.score_compatibility_id.clone(),
        suite: CorpusSuite::Claim,
        cases: results.len(),
        by_category: by_category
            .into_iter()
            .map(|(label, results)| CorpusSliceComparison {
                cases: results.len(),
                comparison: aggregate_benchmark(&results),
                label,
            })
            .collect(),
        by_difficulty: by_difficulty
            .into_iter()
            .map(|(difficulty, results)| CorpusSliceComparison {
                label: format!("{difficulty:?}").to_lowercase(),
                cases: results.len(),
                comparison: aggregate_benchmark(&results),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use crate::{BenchmarkArmResult, Verdict};

    use super::*;

    fn arm(correct: bool) -> BenchmarkArmResult {
        BenchmarkArmResult {
            verdict: Some(if correct {
                Verdict::Accept
            } else {
                Verdict::Unknown
            }),
            claims: 1,
            claims_with_evidence: 1,
            inference_edges: 0,
            verdict_correct: correct,
            evidence_coverage: 1.0,
            unsupported_accepted_claims: 0,
            unsafe_accept: false,
            hidden_assumptions_exposed: 0,
            contradiction_claims_detected: 0,
            counterexamples_detected: 0,
            hard_adversarial_findings: 0,
            soft_adversarial_findings: 0,
            bad_inference_edges_retained: 0,
            deterministic_failure: false,
            deterministic_failure_reason: None,
        }
    }

    fn result(id: &str, correct: bool) -> BenchmarkCaseResult {
        BenchmarkCaseResult {
            fixture_id: id.into(),
            expected_verdict: Verdict::Accept,
            expected_hidden_assumptions: 0,
            expected_contradiction: false,
            expected_counterexample: false,
            baseline: arm(correct),
            harness: arm(correct),
        }
    }

    fn case(id: &str, category: &str, difficulty: CorpusDifficulty) -> CorpusCaseMetadata {
        CorpusCaseMetadata {
            case_id: format!("claim:{id}"),
            fixture_id: id.into(),
            suite: CorpusSuite::Claim,
            fixture_path: format!("{id}.json"),
            category: category.into(),
            difficulty,
            difficulty_rationale: "fixture rationale".into(),
            scoring_mode: CorpusScoringMode::DeterministicOracle,
            provenance: "synthetic_repository_fixture".into(),
            redistribution: CorpusRedistribution::Redistributable,
            contamination_note: "synthetic fixture; exact training exposure unknown".into(),
            status: CorpusCaseStatus::Active,
            superseded_by: None,
        }
    }

    #[test]
    fn claim_aggregation_groups_category_and_difficulty_without_changing_case_identity() {
        let manifest = CorpusManifest {
            manifest_schema_version: 1,
            corpus_version: "1.0.0".into(),
            score_compatibility_id: "corpus-v1".into(),
            description: "test".into(),
            cases: vec![
                case("a", "direct", CorpusDifficulty::Basic),
                case("b", "direct", CorpusDifficulty::Standard),
                case("c", "scope", CorpusDifficulty::Stress),
            ],
        };
        let summary = aggregate_claim_corpus(
            &manifest,
            &[result("a", true), result("b", false), result("c", true)],
        )
        .unwrap();
        assert_eq!(summary.cases, 3);
        assert_eq!(summary.by_category.len(), 2);
        let direct = summary
            .by_category
            .iter()
            .find(|slice| slice.label == "direct")
            .unwrap();
        assert_eq!(direct.cases, 2);
        assert_eq!(direct.comparison.harness.verdict_accuracy, 0.5);
        assert_eq!(summary.by_difficulty.len(), 3);
    }

    #[test]
    fn superseded_case_requires_an_existing_distinct_target() {
        let mut old = case("old", "direct", CorpusDifficulty::Basic);
        old.status = CorpusCaseStatus::Superseded;
        old.superseded_by = Some("claim:new".into());
        let manifest = CorpusManifest {
            manifest_schema_version: 1,
            corpus_version: "1.1.0".into(),
            score_compatibility_id: "corpus-v2".into(),
            description: "test".into(),
            cases: vec![old, case("new", "direct", CorpusDifficulty::Standard)],
        };
        validate_corpus_manifest(&manifest).unwrap();

        let mut invalid = manifest.clone();
        invalid.cases[0].superseded_by = Some("claim:missing".into());
        assert!(matches!(
            validate_corpus_manifest(&invalid),
            Err(CorpusError::SupersededTargetMissing { .. })
        ));
    }

    #[test]
    fn score_compatibility_is_explicit() {
        let base = CorpusManifest {
            manifest_schema_version: 1,
            corpus_version: "1.0.0".into(),
            score_compatibility_id: "corpus-v1".into(),
            description: "test".into(),
            cases: vec![],
        };
        let mut metadata_only = base.clone();
        metadata_only.corpus_version = "1.0.1".into();
        let mut incompatible = base.clone();
        incompatible.score_compatibility_id = "corpus-v2".into();
        assert!(base.score_compatible_with(&metadata_only));
        assert!(!base.score_compatible_with(&incompatible));
    }
}
