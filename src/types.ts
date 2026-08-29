export type EpistemicStatus =
  | 'known'
  | 'supported'
  | 'inferred'
  | 'assumed'
  | 'contradicted'
  | 'unknown';

export interface Evidence {
  id: string;
  source: string;
  locator?: string;
  observation: string;
}

export interface Claim {
  id: string;
  statement: string;
  status: EpistemicStatus;
  evidenceIds: string[];
  assumptionIds: string[];
}

export interface Assumption {
  id: string;
  statement: string;
}

export interface Inference {
  id: string;
  premiseClaimIds: string[];
  conclusionClaimId: string;
  relation: 'deductive' | 'causal' | 'analogical' | 'heuristic';
}

export interface ReasoningArtifact {
  evidence: Evidence[];
  assumptions: Assumption[];
  claims: Claim[];
  inferences: Inference[];
}

export interface Diagnostic {
  code: string;
  severity: 'error' | 'warning';
  message: string;
  subjectId?: string;
}

export interface ValidationResult {
  ok: boolean;
  diagnostics: Diagnostic[];
}
