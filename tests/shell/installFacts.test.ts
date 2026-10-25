import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { FACTS, PHASES, phaseOf } from '../../src/shell/installFacts.ts';

test('every program says what happens at install, use, update and removal', () => {
  for (const [subject, phases] of Object.entries(FACTS)) for (const phase of PHASES) assert.ok(phases[phase.id].length > 0, `${subject}/${phase.id} has no facts`);
});

test('a repair is explained like an update', () => {
  assert.equal(phaseOf('repair'), 'update');
  assert.equal(phaseOf('uninstall'), 'uninstall');
});

// The product installers show the same sentences (packaging/windows/<product>_info.txt); the three repos sit side by side.
for (const [subject, path] of [['hub', 'packaging/windows/hub_info.txt'], ['brain', '../respected-brain/packaging/windows/brain_info.txt'], ['orchestrator', '../respected-orchestrator/packaging/windows/orchestrator_info.txt']] as const) {
  test(`${subject} installer notice carries the Hub's facts`, { skip: !existsSync(path) && 'sibling repository not checked out' }, () => {
    const notice = readFileSync(path, 'utf8');
    for (const phase of PHASES) for (const line of FACTS[subject][phase.id]) assert.ok(notice.includes(line), `${path} lacks: ${line}`);
  });
}
