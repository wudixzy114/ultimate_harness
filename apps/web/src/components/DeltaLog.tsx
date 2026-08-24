import type { Delta } from "../types/delta";

interface Props {
  deltas: Delta[];
}

export function DeltaLog({ deltas }: Props) {
  return (
    <section className="delta-log">
      <header>
        <h2>Delta log</h2>
        <span className="delta-log__count">{deltas.length}</span>
      </header>
      <p className="delta-log__hint">
        Every user interaction is captured as a structured <code>Delta</code> — the
        LLM never re-reads the plan; it only sees the delta.
      </p>
      {deltas.length === 0 ? (
        <p className="delta-log__empty">
          No deltas yet. Click ✓ Approve / ✗ Reject / ✎ Edit on a step, or use
          the plan actions below.
        </p>
      ) : (
        <ol className="delta-log__list">
          {deltas
            .slice()
            .reverse()
            .map((d) => (
              <li key={d.id} className="delta-log__item">
                <code className="delta-log__action">
                  {actionLabel(d)}
                </code>
                <span className="delta-log__time">
                  {new Date(d.timestamp).toLocaleTimeString()}
                </span>
              </li>
            ))}
        </ol>
      )}
    </section>
  );
}

function actionLabel(d: Delta): string {
  if ("plan" in d.target) return `plan · ${d.action.type}`;
  return `step · ${d.action.type}`;
}
