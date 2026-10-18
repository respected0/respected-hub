import test from 'node:test';
import assert from 'node:assert/strict';
import { LATEST_LIMIT, latestRuns } from '../src/views/orchestrator/overviewModel.ts';
import type { Run } from '../src/api.ts';

test('LATEST_LIMIT is the exposed default window for the overview', () => {
  assert.equal(LATEST_LIMIT, 5);
  const runs: Run[] = Array.from({ length: LATEST_LIMIT + 2 }, (_, i) => ({
    run_id: 'run-20260101' + String(i).padStart(6, '0') + '-x',
    status: 'completed',
  }));
  assert.equal(latestRuns(runs).length, LATEST_LIMIT);
});
