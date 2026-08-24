import type { ContextSnapshot } from "../types/context";
import type { WorkingMemory } from "../types/memory";

interface Props {
  snapshot: ContextSnapshot;
  workingMemory: WorkingMemory;
}

/**
 * Visualises the cacheable/variable boundary of the LLM context.
 * The whole point: Working Memory and Step Memory MUST be in the
 * variable suffix, never in the cacheable prefix.
 */
export function ContextPanel({ snapshot, workingMemory }: Props) {
  return (
    <aside className="context">
      <header className="context__header">
        <h2>Context</h2>
        <div className="context__model">
          {snapshot.model.provider} / {snapshot.model.model}
        </div>
      </header>

      <div className="context__bar">
        {snapshot.layers
          .filter((l) => l.is_cacheable)
          .map((l) => (
            <div
              key={l.layer}
              className="context__bar-segment context__bar-segment--cacheable"
              style={{ flex: l.tokens }}
              title={`${l.layer} (${l.tokens} tokens, cacheable)`}
            />
          ))}
        {snapshot.layers
          .filter((l) => !l.is_cacheable)
          .map((l) => (
            <div
              key={l.layer}
              className="context__bar-segment context__bar-segment--variable"
              style={{ flex: l.tokens }}
              title={`${l.layer} (${l.tokens} tokens, variable)`}
            />
          ))}
      </div>

      <div className="context__legend">
        <span>
          <i className="dot dot--cacheable" /> cacheable prefix
        </span>
        <span>
          <i className="dot dot--variable" /> variable suffix
        </span>
      </div>

      <ul className="context__layers">
        {snapshot.layers.map((l) => {
          const pct = Math.round((l.tokens / l.token_budget) * 100);
          return (
            <li key={l.layer} className="context__layer" data-cacheable={l.is_cacheable}>
              <div className="context__layer-name">{l.layer}</div>
              <div className="context__layer-meta">
                {l.tokens}/{l.token_budget} · {l.item_count} items
              </div>
              <div className="context__layer-bar">
                <div className="context__layer-bar-fill" style={{ width: `${pct}%` }} />
              </div>
            </li>
          );
        })}
      </ul>

      <div className="context__wm">
        <h3>Working memory (variable suffix only)</h3>
        <div className="context__wm-line">
          plan: <code>{workingMemory.current_plan_id?.slice(0, 8) ?? "—"}</code>
        </div>
        <div className="context__wm-line">
          step: <code>{workingMemory.current_step_id?.slice(0, 8) ?? "—"}</code>
        </div>
        {workingMemory.hypotheses.length > 0 && (
          <div className="context__wm-section">
            <h4>hypotheses</h4>
            <ul>
              {workingMemory.hypotheses.map((h) => (
                <li key={h.id} data-status={h.status}>
                  <span className="context__wm-confidence">
                    {Math.round(h.confidence * 100)}%
                  </span>
                  {h.text}
                </li>
              ))}
            </ul>
          </div>
        )}
        {workingMemory.decisions.length > 0 && (
          <div className="context__wm-section">
            <h4>decisions</h4>
            <ul>
              {workingMemory.decisions.map((d) => (
                <li key={d.id}>
                  <strong>{d.text}</strong>
                  <span className="context__wm-reason"> — {d.reason}</span>
                </li>
              ))}
            </ul>
          </div>
        )}
        {workingMemory.open_questions.length > 0 && (
          <div className="context__wm-section">
            <h4>open questions</h4>
            <ul>
              {workingMemory.open_questions.map((q) => (
                <li key={q.id} data-blocking={q.blocking}>
                  {q.blocking && <span className="context__wm-blocking">blocking</span>}
                  {q.text}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </aside>
  );
}
