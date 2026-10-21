import test from 'node:test';
import assert from 'node:assert/strict';
import { engine, native } from '../src/shared/api.ts';
import { rppCall, rppTry } from '../src/shared/productClient.ts';
import { listCampaigns, campaignStatus } from '../src/products/orchestrator/campaigns.ts';
import { SECRET_REDACTED } from '../src/shared/rpp.ts';

// Unit tests run on plain Node, outside the Tauri shell, so the native IPC entry
// points must refuse the work instead of pretending it succeeded.
test('native IPC entry points reject outside the desktop app', async () => {
  assert.equal(native, false);
  await assert.rejects(() => engine('list'), /Tauri/);
  await assert.rejects(() => rppCall('secondbrain', 'manifest'), /Tauri/);
  await assert.rejects(() => listCampaigns(), /Tauri/);
  await assert.rejects(() => campaignStatus('campaign-x'), /Tauri/);
});

test('rppTry reports an unreachable product as null instead of throwing', async () => {
  assert.equal(await rppTry('secondbrain', 'health'), null);
});

test('secret values keep a stable redaction marker', () => {
  assert.equal(SECRET_REDACTED, '***set***');
});
