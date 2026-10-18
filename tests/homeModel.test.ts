import { test } from 'node:test';
import assert from 'node:assert/strict';
import { homeMode } from '../src/homeModel.ts';
import type { AppSnapshot } from '../src/homeModel.ts';

const app = (kind: AppSnapshot['kind'], configSource: AppSnapshot['configSource'] = 'file', loading = false): AppSnapshot => ({ kind, configSource, loading });

test('home mode follows real ecosystem state', () => {
  assert.equal(homeMode(false, [app('ok')]), 'preview');
  assert.equal(homeMode(true, []), 'first-launch');
  assert.equal(homeMode(true, [app('ok', 'file', true)]), 'loading');
  assert.equal(homeMode(true, [app('unreachable', null), app('unreachable', null)]), 'first-launch');
  assert.equal(homeMode(true, [app('ok'), app('unreachable', null)]), 'attention');
  assert.equal(homeMode(true, [app('ok', 'defaults'), app('warn')]), 'attention');
  assert.equal(homeMode(true, [app('ok', 'defaults'), app('ok')]), 'setup-incomplete');
  assert.equal(homeMode(true, [app('ok'), app('ok')]), 'active');
});
