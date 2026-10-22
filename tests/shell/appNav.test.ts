import test from 'node:test';
import assert from 'node:assert/strict';
import { APPS, HUB_NAV, SETTINGS_TABS } from '../../src/shell/apps.ts';

test('hub navigation is Home, Discover, Library, Settings', () => {
  assert.deepEqual(HUB_NAV.map(item => item.id), ['home', 'discover', 'apps', 'settings']);
  assert.deepEqual(HUB_NAV.map(item => item.label), ['Ana Sayfa', 'Keşfet', 'Kütüphane', 'Ayarlar']);
});

test('Settings holds the Hub settings and the activity history', () => {
  assert.deepEqual(SETTINGS_TABS.map(tab => tab.id), ['general', 'activity']);
  assert.deepEqual(SETTINGS_TABS.map(tab => tab.label), ['Genel', 'Etkinlik']);
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

test('brain pages are unchanged', () => {
  const brain = APPS.find(app => app.id === 'brain');
  assert.ok(brain, 'brain app is missing');
  assert.deepEqual(brain.pages.map(page => page.id), ['overview', 'memory', 'setup', 'settings']);
});
