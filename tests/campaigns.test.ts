import test from 'node:test';
import assert from 'node:assert/strict';
import { startCampaign, summarizeCampaign } from '../src/campaigns.ts';

test('summary: a running campaign with a live supervisor polls as running', () => {
  const summary = summarizeCampaign({
    campaign_id: 'camp-20260101000000-aaaaaaaa', status: 'running', result: null, supervisor_alive: true,
    events: [{ event: 'lane.started', at: '2026-01-01T00:00:00Z', lane: 'native', run_id: 'run-1' }],
  });
  assert.equal(summary.phase, 'running');
  assert.ok(summary.headline.includes('çalışıyor'), summary.headline);
  assert.equal(summary.lanes.length, 0);
  assert.equal(summary.timeline.length, 1);
  assert.ok(summary.timeline[0].text.includes('native'), summary.timeline[0].text);
  assert.equal(summary.chain, undefined);
});

test('summary: approved lanes translate status, tokens and detail', () => {
  const summary = summarizeCampaign({
    campaign_id: 'camp-20260101000000-aaaaaaaa', status: 'LANES_APPROVED', supervisor_alive: false,
    result: {
      campaign_id: 'camp-20260101000000-aaaaaaaa', status: 'LANES_APPROVED',
      lanes: [
        { name: 'native', status: 'MASTER_APPROVED', token_usage: 1200, changed_files: ['a.ts', 'b.ts'] },
        { name: 'view', status: 'SKIPPED', reason: 'dependency native not approved', attempts: 1 },
      ],
    },
    events: [
      { event: 'lane.completed', at: '2026-01-01T00:00:00Z', lane: 'native', status: 'MASTER_APPROVED' },
      { event: 'master.finished', at: '2026-01-01T00:05:00Z', status: 'COMPLETE' },
    ],
  });
  assert.equal(summary.phase, 'finished');
  assert.ok(summary.headline.includes('tamamlandı'), summary.headline);
  assert.equal(summary.lanes.length, 2);
  assert.equal(summary.lanes[0].name, 'native');
  assert.equal(summary.lanes[0].statusText, 'Onaylandı');
  assert.equal(summary.lanes[0].tokens, 1200);
  assert.ok(summary.lanes[0].detail.includes('2 dosya'), summary.lanes[0].detail);
  assert.equal(summary.lanes[1].statusText, 'Atlandı');
  assert.ok(summary.lanes[1].detail.includes('Bağımlı şerit'), summary.lanes[1].detail);
});

test('summary: needs review, blocked and interrupted phases', () => {
  const review = summarizeCampaign({ status: 'LANES_NEED_REVIEW', result: { status: 'LANES_NEED_REVIEW' }, events: [] });
  assert.equal(review.phase, 'needs_review');
  const blocked = summarizeCampaign({ status: 'FAILED', result: { status: 'FAILED' }, supervisor_alive: false, events: [] });
  assert.equal(blocked.phase, 'blocked');
  const interrupted = summarizeCampaign({ status: 'interrupted', result: null, supervisor_alive: false, events: [] });
  assert.equal(interrupted.phase, 'interrupted');
  assert.ok(interrupted.headline.includes('yarıda'), interrupted.headline);
});

test('summary: timeline texts are plain Turkish', () => {
  const summary = summarizeCampaign({
    status: 'LANES_NEED_REVIEW', result: { status: 'LANES_NEED_REVIEW' },
    events: [
      { event: 'campaign.completed', at: 't1', status: 'LANES_NEED_REVIEW' },
      { event: 'lane.completed', at: 't2', lane: 'view', status: 'SKIPPED' },
    ],
  });
  assert.ok(summary.timeline[0].text.includes('inceleme bekliyor'), summary.timeline[0].text);
  assert.ok(summary.timeline[1].text.includes('view') && summary.timeline[1].text.includes('Atlandı'), summary.timeline[1].text);
  assert.equal(summary.timeline[0].at, 't1');
});

test('summary: tolerates older campaigns without chain or lanes', () => {
  const empty = summarizeCampaign(null);
  assert.equal(empty.phase, 'interrupted');
  assert.deepEqual(empty.lanes, []);
  assert.deepEqual(empty.timeline, []);
  assert.equal(empty.chain, undefined);
  assert.equal(empty.finalText, undefined);

  const partial = summarizeCampaign({ status: 'LANES_APPROVED', result: { lanes: [{}] }, events: 'nope' });
  assert.equal(partial.lanes.length, 1);
  assert.equal(partial.lanes[0].name, '(isimsiz şerit)');
  assert.equal(partial.lanes[0].tokens, null);
  assert.deepEqual(partial.timeline, []);
});

test('summary: surfaces chain and final text only when present', () => {
  const summary = summarizeCampaign({
    status: 'running', supervisor_alive: true, events: [],
    result: { chain: { index: 2, parent: 'camp-20260101000000-aaaaaaaa', outcome: 'continued' }, final_text: 'Özet metni.' },
  });
  assert.deepEqual(summary.chain, { index: 2, parent: 'camp-20260101000000-aaaaaaaa', outcome: 'continued' });
  assert.equal(summary.finalText, 'Özet metni.');
});

test('startCampaign rejects non-object specs before touching the engine', async () => {
  await assert.rejects(() => startCampaign('not json'), /JSON/);
  await assert.rejects(() => startCampaign('[1,2]'), /nesne/);
  await assert.rejects(() => startCampaign('null'), /nesne/);
});