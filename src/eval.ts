import type { ReasoningArtifact } from './types.js';
import { validateArtifact } from './validate.js';

export interface ArtifactMetrics {
  valid: boolean;
  evidenceCoverage: number;
  explicitUnknownRate: number;
  acceptedWithoutEvidence: number;
}

export function scoreArtifact(artifact: ReasoningArtifact): ArtifactMetrics {
  const validation = validateArtifact(artifact);
  const checkable = artifact.claims.filter((claim) => claim.status !== 'unknown' && claim.status !== 'assumed');
  const evidenced = checkable.filter((claim) => claim.evidenceIds.length > 0);
  const acceptedWithoutEvidence = artifact.claims.filter(
    (claim) => (claim.status === 'known' || claim.status === 'supported') && claim.evidenceIds.length === 0,
  ).length;
  return {
    valid: validation.ok,
    evidenceCoverage: checkable.length === 0 ? 1 : evidenced.length / checkable.length,
    explicitUnknownRate: artifact.claims.length === 0 ? 0 : artifact.claims.filter((claim) => claim.status === 'unknown').length / artifact.claims.length,
    acceptedWithoutEvidence,
  };
}
