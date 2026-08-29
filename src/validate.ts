import type { Diagnostic, ReasoningArtifact, ValidationResult } from './types.js';

function collectDuplicateIds(items: ReadonlyArray<{ id: string }>, kind: string): Diagnostic[] {
  const seen = new Set<string>();
  const diagnostics: Diagnostic[] = [];
  for (const item of items) {
    if (seen.has(item.id)) {
      diagnostics.push({ code: 'duplicate_id', severity: 'error', message: `Duplicate ${kind} id: ${item.id}`, subjectId: item.id });
    }
    seen.add(item.id);
  }
  return diagnostics;
}

export function validateArtifact(artifact: ReasoningArtifact): ValidationResult {
  const diagnostics: Diagnostic[] = [
    ...collectDuplicateIds(artifact.evidence, 'evidence'),
    ...collectDuplicateIds(artifact.assumptions, 'assumption'),
    ...collectDuplicateIds(artifact.claims, 'claim'),
    ...collectDuplicateIds(artifact.inferences, 'inference'),
  ];

  const evidenceIds = new Set(artifact.evidence.map((item) => item.id));
  const assumptionIds = new Set(artifact.assumptions.map((item) => item.id));
  const claimIds = new Set(artifact.claims.map((item) => item.id));

  for (const claim of artifact.claims) {
    for (const evidenceId of claim.evidenceIds) {
      if (!evidenceIds.has(evidenceId)) diagnostics.push({ code: 'missing_evidence', severity: 'error', message: `Claim ${claim.id} references missing evidence ${evidenceId}`, subjectId: claim.id });
    }
    for (const assumptionId of claim.assumptionIds) {
      if (!assumptionIds.has(assumptionId)) diagnostics.push({ code: 'missing_assumption', severity: 'error', message: `Claim ${claim.id} references missing assumption ${assumptionId}`, subjectId: claim.id });
    }
    if ((claim.status === 'known' || claim.status === 'supported') && claim.evidenceIds.length === 0) {
      diagnostics.push({ code: 'unproven_claim', severity: 'error', message: `${claim.status} claim ${claim.id} has no evidence`, subjectId: claim.id });
    }
    if (claim.status === 'unknown' && claim.evidenceIds.length > 0) {
      diagnostics.push({ code: 'unknown_with_evidence', severity: 'warning', message: `Unknown claim ${claim.id} carries evidence; consider inferred/supported status`, subjectId: claim.id });
    }
  }

  for (const inference of artifact.inferences) {
    if (!claimIds.has(inference.conclusionClaimId)) diagnostics.push({ code: 'missing_conclusion', severity: 'error', message: `Inference ${inference.id} references missing conclusion ${inference.conclusionClaimId}`, subjectId: inference.id });
    for (const premiseId of inference.premiseClaimIds) {
      if (!claimIds.has(premiseId)) diagnostics.push({ code: 'missing_premise', severity: 'error', message: `Inference ${inference.id} references missing premise ${premiseId}`, subjectId: inference.id });
    }
  }

  return { ok: !diagnostics.some((item) => item.severity === 'error'), diagnostics };
}
