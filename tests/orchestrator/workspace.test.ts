import test from 'node:test';
import assert from 'node:assert/strict';
import { APPS } from '../../src/shell/apps.ts';
import { ORCHESTRATOR_PAGES, mountFor, dataRootSummary } from '../../src/products/orchestrator/views/workspaceModel.ts';

test('orchestrator page order/ids match the workspace model registry', () => {
  const orchestrator = APPS.find(app => app.id === 'orchestrator');
  assert.ok(orchestrator, 'orchestrator app is missing');
  assert.deepEqual(orchestrator.pages.map(page => page.id), ORCHESTRATOR_PAGES.map(page => page.id));
  assert.deepEqual(orchestrator.pages.map(page => page.label), ORCHESTRATOR_PAGES.map(page => page.label));
});

test('every orchestrator page id has a mount', () => {
  for (const page of ORCHESTRATOR_PAGES) {
    assert.notEqual(mountFor('orchestrator', page.id), 'none', `${page.id} has no mount`);
  }
});

test('secondbrain mounts stay overview/setup/settings', () => {
  assert.equal(mountFor('secondbrain', 'overview'), 'overview');
  assert.equal(mountFor('secondbrain', 'setup'), 'setup');
  assert.equal(mountFor('secondbrain', 'memory'), 'memory');
  assert.equal(mountFor('secondbrain', 'settings'), 'settings');
  assert.equal(mountFor('secondbrain', 'agents'), 'none');
});

test('unknown app or page has no mount', () => {
  assert.equal(mountFor('unknown', 'overview'), 'none');
  assert.equal(mountFor('orchestrator', 'nope'), 'none');
});

test('data-root summary: zero campaigns names the directory and the settings page', () => {
  const summary = dataRootSummary('C:/data/root', 0, true);
  assert.equal(summary.dataRoot, 'C:/data/root');
  assert.equal(summary.campaignCount, 0);
  assert.match(summary.text, /kampanya bulunamad\u0131/);
  assert.match(summary.text, /Ba\u011flant\u0131lar ve ayarlar/);
});

test('data-root summary: n campaigns reports the real count', () => {
  const summary = dataRootSummary('C:/data/root', 3, true);
  assert.equal(summary.campaignCount, 3);
  assert.match(summary.text, /3 kampanya/);
});

test('data-root summary: browser mode is honest and claims no engine data', () => {
  const summary = dataRootSummary('', null, false);
  assert.equal(summary.campaignCount, null);
  assert.match(summary.text, /masa\u00fcst\u00fc/);
});

test('data-root summary: unknown root is reported, not faked', () => {
  const summary = dataRootSummary('', 0, true);
  assert.equal(summary.dataRoot, '');
  assert.match(summary.text, /yap\u0131land\u0131r\u0131lmad\u0131/);
});