import test from 'node:test';
import assert from 'node:assert/strict';
import type { Project, ResolvedEntry, Resolution, ScanIssue } from '../../src/shared/integrationsTypes.ts';
import { projectBindings, projectIssue } from '../../src/shell/projectScope.ts';

const entry = (over: Partial<ResolvedEntry> & Pick<ResolvedEntry, 'asset_key' | 'name' | 'kind' | 'client' | 'scope'>): ResolvedEntry => ({
  project: null, source_path: 'C:/Users/test/.config.json', configured: true, effective: true,
  enabled: true, managed: false, owner: { kind: 'user' }, state: 'effective', ...over,
});
const project = (id: string, root: string): Project => ({ id, root });
const p1 = project('p1', 'C:/work/alpha'), p2 = project('p2', 'C:/work/beta'), p3 = project('p3', 'C:/work/gamma');
const report = (entries: ResolvedEntry[], issues: ScanIssue[] = []): Resolution => ({ entries, issues, untrusted: [] });

const globalPg = entry({ asset_key: 'mcp:pg-global', name: 'postgres', kind: 'mcp', client: 'claude', scope: 'global' });
const p1Pg = entry({ asset_key: 'mcp:pg-p1', name: 'postgres', kind: 'mcp', client: 'claude', scope: 'project', project: p1 });
const p2Beta = entry({ asset_key: 'mcp:beta', name: 'beta', kind: 'mcp', client: 'codex', scope: 'project', project: p2 });
const p3Gamma = entry({ asset_key: 'skill:gamma', name: 'gamma', kind: 'skill', client: 'cursor', scope: 'project', project: p3 });

test('three registered projects keep distinct effective bindings', () => {
  const value = report([globalPg, p1Pg, p2Beta, p3Gamma]);
  assert.deepEqual(projectBindings(value, p1.id).map(item => item.asset_key), ['mcp:pg-p1']);
  assert.deepEqual(projectBindings(value, p2.id).map(item => item.asset_key), ['mcp:pg-global', 'mcp:beta']);
  assert.deepEqual(projectBindings(value, p3.id).map(item => item.asset_key), ['mcp:pg-global', 'skill:gamma']);
});

test('project MCP shadows the global override inside that project only', () => {
  const value = report([globalPg, p1Pg]);
  assert.ok(projectBindings(value, p1.id).every(item => item.scope === 'project'));
  assert.deepEqual(projectBindings(value, p2.id).map(item => item.asset_key), ['mcp:pg-global']);
});

test('non-effective and missing-project issues stay out of the binding list', () => {
  const overridden = entry({ asset_key: 'mcp:old', name: 'old', kind: 'mcp', client: 'claude', scope: 'global', state: 'overridden', effective: false });
  const missing = projectIssue(p1, [{ code: 'project_missing', message: 'bulunamadı', source_path: p1.root }]);
  assert.equal(projectBindings(report([overridden]), p1.id).length, 0);
  assert.equal(missing?.code, 'project_missing');
  assert.equal(projectIssue(p2, [{ code: 'project_missing', message: 'x', source_path: p1.root }]), null);
});
