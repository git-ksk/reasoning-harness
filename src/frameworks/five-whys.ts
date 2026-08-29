import type { Diagnostic } from '../types.js';

export interface WhyLink {
  effect: string;
  cause: string;
  evidenceIds: string[];
}

export interface FiveWhysTrace {
  symptom: string;
  links: WhyLink[];
  rootCause: string;
}

export function validateFiveWhys(trace: FiveWhysTrace): Diagnostic[] {
  const diagnostics: Diagnostic[] = [];
  if (trace.links.length < 2) diagnostics.push({ code: 'why_chain_too_shallow', severity: 'warning', message: 'Why chain has fewer than two causal links' });
  for (const [index, link] of trace.links.entries()) {
    if (link.effect.trim() === link.cause.trim()) diagnostics.push({ code: 'why_restates_effect', severity: 'error', message: `Why link ${index + 1} restates its effect as the cause` });
    if (link.evidenceIds.length === 0) diagnostics.push({ code: 'why_without_evidence', severity: 'warning', message: `Why link ${index + 1} has no evidence` });
  }
  const last = trace.links.at(-1);
  if (last && last.cause.trim() !== trace.rootCause.trim()) diagnostics.push({ code: 'root_cause_mismatch', severity: 'error', message: 'Root cause does not match the final causal link' });
  return diagnostics;
}
