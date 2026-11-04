import test from 'node:test';
import assert from 'node:assert/strict';
import type { CatalogEntry } from '../../src/shared/integrationsTypes.ts';
import { catalogCategories, filterCatalog } from '../../src/shell/thirdPartyCatalog.ts';

const entries: CatalogEntry[] = [
  { id: 'a', name: 'GitHub MCP', type: 'mcp', description: 'Depo ve issue erişimi', category: 'Geliştirme',
    source_url: 'https://github.com/github/github-mcp-server', supported_clients: ['claude', 'codex', 'open_code'], license: 'MIT', provenance: null },
  { id: 'b', name: 'Güvenlik Taraması', type: 'plugin', description: 'Yapılandırma denetimi', category: 'Güvenlik',
    source_url: 'https://github.com/example/security', supported_clients: ['claude'], license: null, provenance: 'topluluk' },
  { id: 'c', name: 'Playwright Skill', type: 'skill', description: 'Tarayıcı testleri', category: 'Test',
    source_url: 'https://github.com/microsoft/playwright', supported_clients: ['codex', 'gemini'], license: 'Apache-2.0', provenance: null },
];

test('filters match the fixture exactly by kind, category and client', () => {
  assert.deepEqual(filterCatalog(entries, { kind: 'skill' }).map(entry => entry.id), ['c']);
  assert.deepEqual(filterCatalog(entries, { category: 'Güvenlik' }).map(entry => entry.id), ['b']);
  assert.deepEqual(filterCatalog(entries, { client: 'open_code' }).map(entry => entry.id), ['a']);
  assert.deepEqual(filterCatalog(entries, { kind: 'mcp', client: 'codex', category: 'Geliştirme' }).map(entry => entry.id), ['a']);
  assert.deepEqual(filterCatalog(entries, { kind: 'rule' }).map(entry => entry.id), []);
});

test('search is Turkish- and Unicode-aware and trims the query', () => {
  assert.deepEqual(filterCatalog(entries, { query: '  YAPILANDIRMA ' }).map(entry => entry.id), ['b']);
  assert.deepEqual(filterCatalog(entries, { query: 'güvenlik' }).map(entry => entry.id), ['b']);
  assert.deepEqual(filterCatalog(entries, { query: 'tarayıcı' }).map(entry => entry.id), ['c']);
  assert.deepEqual(filterCatalog(entries, { query: 'yok-böyle-kayıt' }).map(entry => entry.id), []);
});

test('categories are unique and sorted for Turkish readers', () => {
  assert.deepEqual(catalogCategories(entries), ['Geliştirme', 'Güvenlik', 'Test']);
  assert.deepEqual(catalogCategories([]), []);
});

test('filtering stays exact across 100 catalog records', () => {
  const many: CatalogEntry[] = Array.from({ length: 100 }, (_, index) => ({
    id: `e${index}`, name: index === 73 ? 'Special Server' : `Server ${index}`, type: index % 2 ? 'mcp' : 'skill',
    description: 'demo', category: index % 3 ? 'Dev' : 'Test', source_url: 'https://github.com/example/demo',
    supported_clients: index % 4 ? ['claude'] : ['codex'], license: null, provenance: null,
  }));
  assert.deepEqual(filterCatalog(many, { query: 'special server' }).map(entry => entry.id), ['e73']);
  assert.equal(filterCatalog(many, { client: 'codex' }).length, 25);
  assert.equal(filterCatalog(many, { kind: 'mcp', category: 'Test' }).length, 17);
});
