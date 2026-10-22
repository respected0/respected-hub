import { test } from 'node:test';
import assert from 'node:assert/strict';
import { homeMode } from '../../src/shell/homeModel.ts';
import type { AppSnapshot } from '../../src/shell/homeModel.ts';

const app = (kind: AppSnapshot['kind'], loading = false): AppSnapshot => ({ kind, loading });

test('home mode follows real ecosystem state', () => {
  assert.equal(homeMode(false, [app('ok')]), 'preview');
  assert.equal(homeMode(true, []), 'first-launch');
  assert.equal(homeMode(true, [app('ok', true)]), 'loading');
  assert.equal(homeMode(true, [app('unreachable'), app('unreachable')]), 'first-launch');
  assert.equal(homeMode(true, [app('ok'), app('unreachable')]), 'attention');
  assert.equal(homeMode(true, [app('ok'), app('warn')]), 'attention');
  assert.equal(homeMode(true, [app('ok'), app('ok')]), 'active', 'an installed product is set up: its installer ran its setup');
});
