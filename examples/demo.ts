import { ReasoningHarness, scoreArtifact, type ReasoningArtifact } from '../src/index.js';

const artifact: ReasoningArtifact = {
  evidence: [{ id: 'e1', source: 'example', observation: 'A deterministic source reports value A.' }],
  assumptions: [{ id: 'a1', statement: 'The source remained authoritative during the observation window.' }],
  claims: [
    { id: 'c1', statement: 'Value A was observed.', status: 'known', evidenceIds: ['e1'], assumptionIds: [] },
    { id: 'c2', statement: 'Value A will remain stable.', status: 'assumed', evidenceIds: [], assumptionIds: ['a1'] },
    { id: 'c3', statement: 'Future behavior is guaranteed.', status: 'unknown', evidenceIds: [], assumptionIds: [] },
  ],
  inferences: [],
};

const result = await new ReasoningHarness().run(artifact);
console.log(JSON.stringify({ validation: result.validation, metrics: scoreArtifact(result.artifact) }, null, 2));
