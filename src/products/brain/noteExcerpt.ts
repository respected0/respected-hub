// A short, readable piece of a Markdown note for a card: front matter, heading marks and list bullets dropped.
export function excerpt(markdown: string, lines = 5, width = 150): string[] {
  const body = markdown.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n/, '');
  return body.split(/\r?\n/)
    .map(line => line.replace(/^\s*(#{1,6}\s+|[-*]\s+|>\s*)/, '').replace(/\*\*|`|<!--.*?-->/g, '').trim())
    .filter(line => line && !/^[-|:\s]+$/.test(line))
    .slice(0, lines)
    .map(line => line.length > width ? `${line.slice(0, width - 1)}…` : line);
}
