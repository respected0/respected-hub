import { CLIENTS, toggle } from '../brainClients.ts';

// One switch per supported AI client. Clients not found on this computer cannot be newly selected (their folders would be created).
export default function ClientPicker({ kind, selected, onChange, detected, disabled = false }: {
  kind: 'hooks' | 'mcp'; selected: string[]; onChange: (next: string[]) => void; detected: Record<string, boolean> | null; disabled?: boolean;
}) {
  return <div role="group" aria-label={kind === 'hooks' ? 'Hafıza kaynağı AI araçları' : 'MCP erişimi verilecek AI araçları'}>
    {CLIENTS.filter(client => client[kind]).map(client => {
      const found = detected ? detected[client.id] === true : true;
      const on = selected.includes(client.id);
      return <label className="choice" key={client.id} data-client={client.id}><span><strong>{client.label}</strong>
        <small>{detected === null ? 'Aranıyor…' : found ? 'Bu bilgisayarda bulundu' : 'Bu bilgisayarda bulunamadı'}</small></span>
        <input type="checkbox" role="switch" checked={on} disabled={disabled || (!found && !on)} onChange={event => onChange(toggle(selected, client.id, event.target.checked))} /></label>;
    })}
  </div>;
}
