import type { Project, Resolution, ResolvedEntry, ScanIssue } from '../shared/integrationsTypes.ts';

/** Only the resolver's effective entries are shown; a missing root is a safe issue, not an empty project. */
export function projectIssue(project: Project, issues: ScanIssue[]): ScanIssue | null {
  return issues.find(issue => issue.code === 'project_missing' && issue.source_path === project.root) ?? null;
}

/** Effective bindings for one project: its own entries plus unshadowed global MCP/plugin records. */
export function projectBindings(report: Resolution, projectId: string): ResolvedEntry[] {
  const inScope = report.entries.filter(entry => entry.state === 'effective'
    && (entry.scope === 'global' || (entry.scope === 'project' && entry.project?.id === projectId)));
  const shadowed = new Set(inScope
    .filter(entry => entry.scope === 'project' && (entry.kind === 'mcp' || entry.kind === 'plugin'))
    .map(entry => `${entry.kind}:${entry.name}`));
  return inScope.filter(entry => !(entry.scope === 'global'
    && (entry.kind === 'mcp' || entry.kind === 'plugin') && shadowed.has(`${entry.kind}:${entry.name}`)));
}
