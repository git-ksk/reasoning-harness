import type { ReasoningArtifact, ValidationResult } from './types.js';
import { validateArtifact } from './validate.js';

export interface HarnessPass {
  readonly name: string;
  run(artifact: ReasoningArtifact): Promise<ReasoningArtifact> | ReasoningArtifact;
}

export interface HarnessRunResult {
  artifact: ReasoningArtifact;
  validation: ValidationResult;
  passes: string[];
}

export class ReasoningHarness {
  constructor(private readonly passes: readonly HarnessPass[] = []) {}

  async run(initial: ReasoningArtifact): Promise<HarnessRunResult> {
    let artifact = initial;
    const executed: string[] = [];
    for (const pass of this.passes) {
      artifact = await pass.run(artifact);
      executed.push(pass.name);
      const validation = validateArtifact(artifact);
      if (!validation.ok) return { artifact, validation, passes: executed };
    }
    return { artifact, validation: validateArtifact(artifact), passes: executed };
  }
}
