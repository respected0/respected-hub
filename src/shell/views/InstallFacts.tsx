import { FACTS, PHASES, type FactPhase, type FactSubject } from '../installFacts.ts';

// What a program installs, changes and removes: always shown, never collapsed. `only` shows one phase (the action being confirmed); without it every phase is listed.
export default function InstallFacts({ subject, only }: { subject: FactSubject; only?: FactPhase }) {
  const phases = PHASES.filter(phase => !only || phase.id === only);
  return <div className="facts">
    <h3>{only ? 'Neler değişecek?' : 'Neler kurulur, güncellenir, kaldırılır?'}</h3>
    <div className="facts-grid">{phases.map(phase => <section key={phase.id} aria-label={phase.label}>
      <h4>{phase.label}</h4><ul>{FACTS[subject][phase.id].map(line => <li key={line}>{line}</li>)}</ul></section>)}</div>
  </div>;
}
