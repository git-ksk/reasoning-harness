export interface StructuredModelRequest {
  task: string;
  input: unknown;
  schemaName: string;
}

export interface ModelAdapter {
  readonly id: string;
  generateStructured<T>(request: StructuredModelRequest): Promise<T>;
}

/**
 * Model adapters are intentionally outside the correctness boundary.
 * Their output is always treated as a candidate and must pass deterministic
 * validation before the harness may accept it.
 */
