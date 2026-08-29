import assert from 'node:assert/strict';
import test from 'node:test';
import { scoreArtifact, validateArtifact, type ReasoningArtifact } from '../src/index.js';

const good: ReasoningArtifact = {
  evidence: [{ id: 'e1', source: 'fixture', observation: 'The system reads provider time.' }],
  assumptions: [],
  claims: [{ id: 'c1', statement: 'Provider time is read.', status: 'known', evidenceIds: ['e1'], assumptionIds: [] }],
  inferences: [],
};

test('accepts an evidence-backed known claim', () => {
  assert.equal(validateArtifact(good).ok, true);
  assert.equal(scoreArtifact(good).evidenceCoverage, 1);
});

test('rejects a supported claim without evidence', () => {
  const artifact = structuredClone(good);
  artifact.claims[0]!.evidenceIds = [];
  artifact.claims[0]!.status = 'supported';
  const result = validateArtifact(artifact);
  assert.equal(result.ok, false);
  assert.ok(result.diagnostics.some((item) => item.code === 'unproven_claim'));
});

test('rejects references to missing evidence', () => {
  const artifact = structuredClone(good);
  artifact.claims[0]!.evidenceIds = ['missing'];
  const result = validateArtifact(artifact);
  assert.equal(result.ok, false);
  assert.ok(result.diagnostics.some((item) => item.code === 'missing_evidence'));
});
