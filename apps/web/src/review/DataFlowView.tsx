import type { DataFlow } from "./types";

// Renders a DataFlow as a small grid: input / output / side-effect / depends-on.
export function DataFlowView({ df }: { df: DataFlow }) {
  return (
    <div className="review-df">
      {df.input ? (
        <div className="review-df__row">
          <span className="review-df__k">input</span>
          <code>{df.input}</code>
        </div>
      ) : null}
      {df.output ? (
        <div className="review-df__row">
          <span className="review-df__k">output</span>
          <code>{df.output}</code>
        </div>
      ) : null}
      {df.side_effects && df.side_effects.length > 0 ? (
        <div className="review-df__row">
          <span className="review-df__k">side-effect</span>
          <ul>
            {df.side_effects.map((s, i) => (
              <li key={i}>{s}</li>
            ))}
          </ul>
        </div>
      ) : null}
      {df.depends_on && df.depends_on.length > 0 ? (
        <div className="review-df__row">
          <span className="review-df__k">depends-on</span>
          <ul>
            {df.depends_on.map((s, i) => (
              <li key={i}>{s}</li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  );
}
