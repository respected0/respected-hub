import test from 'node:test';
import assert from 'node:assert/strict';
import { APPS, HUB_NAV } from '../src/apps.ts';

test('hub navigation exposes exactly the six product pages in order', () => {
  assert.deepEqual(HUB_NAV.map(item => item.id), ['home', 'apps', 'discover', 'connections', 'activity', 'settings']);
  assert.deepEqual(HUB_NAV.map(item => item.label), ['Ana Sayfa', 'Uygulamalar\u0131m', 'Ke\u015ffet', '\u00dcr\u00fcn durumu', 'Etkinlik', 'Ayarlar']);
});

test('orchestrator exposes the mounted workspace pages in order', () => {
  const orchestrator = APPS.find(app => app.id === 'orchestrator');
  assert.ok(orchestrator, 'orchestrator app is missing');
  assert.deepEqual(orchestrator.pages.map(page => page.id),
    ['overview', 'agents', 'evidence', 'campaigns', 'runs', 'settings']);
  assert.equal(orchestrator.pages.find(page => page.id === 'agents')?.label, 'Ajanlar ve modeller');
  assert.equal(orchestrator.pages.find(page => page.id === 'evidence')?.label, 'Kan\u0131tlar');
  assert.equal(orchestrator.pages.find(page => page.id === 'campaigns')?.label, 'Kampanyalar');
  assert.equal(orchestrator.pages.find(page => page.id === 'runs')?.label, 'G\u00f6revler');
});

test('secondbrain pages are unchanged', () => {
  const secondbrain = APPS.find(app => app.id === 'secondbrain');
  assert.ok(secondbrain, 'secondbrain app is missing');
  assert.deepEqual(secondbrain.pages.map(page => page.id), ['overview', 'memory', 'setup', 'settings']);
});
