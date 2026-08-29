import assert from 'node:assert/strict';
import test from 'node:test';
import { validateFiveWhys } from '../src/index.js';

test('flags a circular restatement instead of accepting decorative why analysis', () => {
  const result = validateFiveWhys({
    symptom: 'request failed',
    links: [
      { effect: 'request failed', cause: 'request failed', evidenceIds: [] },
      { effect: 'request failed', cause: 'timeout policy rejected it', evidenceIds: ['e1'] },
    ],
    rootCause: 'timeout policy rejected it',
  });
  assert.ok(result.some((item) => item.code === 'why_restates_effect'));
});
