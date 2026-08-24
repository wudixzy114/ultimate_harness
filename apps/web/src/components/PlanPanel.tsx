import { useState } from "react";
import type { Plan, Step, StepIntent, StepStatus } from "../types/plan";
import type { Delta, DeltaAction } from "../types/delta";

const INTENT_LABEL: Record<StepIntent, string> = {
  investigate: "Investigate",
  implement: "Implement",
  verify: "Verify",
  ask_user: "Ask User",
  review: "Review",
  communicate: "Communicate",
};

const INTENT_ICON: Record<StepIntent, string> = {
  investigate: "🔍",
  implement: "✏️",
  verify: "✓",
  ask_user: "❓",
  review: "👁",
  communicate: "💬",
};

const STATUS_COLOR: Record<StepStatus, string> = {
  pending: "var(--c-pending)",
  in_progress: "var(--c-progress)",
  done: "var(--c-done)",
  failed: "var(--c-failed)",
  skipped: "var(--c-skipped)",
  blocked: "var(--c-blocked)",
};

interface Props {
  plan: Plan;
  onDelta?: (delta: Delta) => void;
}

export function PlanPanel({ plan, onDelta }: Props) {
  return (
    <article className="plan">
      <header className="plan__header">
        <h1 className="plan__title">{plan.title}</h1>
        <div className="plan__meta">
          <span className="plan__status" data-status={plan.status}>
            {plan.status.replace("_", " ")}
          </span>
          <code className="plan__id">id: {plan.id.slice(0, 8)}</code>
        </div>
      </header>

      <section className="plan__section">
        <h2>Task</h2>
        <p className="plan__task">{plan.task}</p>
      </section>

      <section className="plan__section">
        <h2>Goal</h2>
        <p className="plan__goal">{plan.goal}</p>
      </section>

      {plan.success_criteria.length > 0 && (
        <section className="plan__section">
          <h2>
            Success criteria{" "}
            <span className="plan__count">
              {plan.success_criteria.filter((c) => c.verified).length}/
              {plan.success_criteria.length}
            </span>
          </h2>
          <ul className="plan__list">
            {plan.success_criteria.map((c) => (
              <li key={c.id} data-verified={c.verified}>
                <span className="plan__checkbox">
                  {c.verified ? "☑" : "☐"}
                </span>
                {c.text}
              </li>
            ))}
          </ul>
        </section>
      )}

      {plan.assumptions.length > 0 && (
        <section className="plan__section">
          <h2>Assumptions</h2>
          <ul className="plan__list">
            {plan.assumptions.map((a) => (
              <li key={a.id} data-critical={a.critical}>
                {a.critical && <span className="plan__badge">critical</span>}
                {a.text}
              </li>
            ))}
          </ul>
        </section>
      )}

      {plan.unknowns.length > 0 && (
        <section className="plan__section">
          <h2>Unknowns</h2>
          <ul className="plan__list">
            {plan.unknowns.map((u) => (
              <li key={u.id} data-blocking={u.blocking}>
                {u.blocking && <span className="plan__badge">blocking</span>}
                {u.text}
              </li>
            ))}
          </ul>
        </section>
      )}

      <section className="plan__section plan__section--steps">
        <h2>
          Steps <span className="plan__count">{plan.steps.length}</span>
        </h2>
        <ol className="steps">
          {plan.steps.map((step) => (
            <StepCard
              key={step.id}
              step={step}
              onDelta={(action) => {
                if (!onDelta) return;
                onDelta({
                  id: crypto.randomUUID(),
                  timestamp: new Date().toISOString(),
                  target: { step: { plan_id: plan.id, step_id: step.id } },
                  action,
                  comment: null,
                });
              }}
            />
          ))}
        </ol>
      </section>

      {onDelta && (
        <section className="plan__section plan__section--actions">
          <h2>Plan actions</h2>
          <div className="plan__buttons">
            <button
              className="btn btn--primary"
              onClick={() =>
                onDelta({
                  id: crypto.randomUUID(),
                  timestamp: new Date().toISOString(),
                  target: { plan: plan.id },
                  action: { type: "approve_plan" } satisfies DeltaAction,
                  comment: null,
                })
              }
            >
              ✓ Approve plan
            </button>
            <button
              className="btn"
              onClick={() => {
                const reason = prompt("Reason for rejection?");
                if (!reason) return;
                onDelta({
                  id: crypto.randomUUID(),
                  timestamp: new Date().toISOString(),
                  target: { plan: plan.id },
                  action: { type: "reject_plan", reason } satisfies DeltaAction,
                  comment: null,
                });
              }}
            >
              ✗ Reject plan
            </button>
            <button
              className="btn btn--danger"
              onClick={() => {
                const reason = prompt("Reason to abort?");
                if (!reason) return;
                onDelta({
                  id: crypto.randomUUID(),
                  timestamp: new Date().toISOString(),
                  target: { plan: plan.id },
                  action: { type: "abort_plan", reason } satisfies DeltaAction,
                  comment: null,
                });
              }}
            >
              ⏹ Abort
            </button>
          </div>
        </section>
      )}
    </article>
  );
}

interface StepProps {
  step: Step;
  onDelta: (action: DeltaAction) => void;
}

function StepCard({ step, onDelta }: StepProps) {
  const [editing, setEditing] = useState(false);

  const startEditing = () => setEditing(true);

  return (
    <li
      className="step"
      data-status={step.status}
      style={{ borderLeftColor: STATUS_COLOR[step.status] }}
    >
      <div className="step__head">
        <span className="step__order">{step.order + 1}</span>
        <span className="step__icon" aria-hidden>
          {INTENT_ICON[step.intent]}
        </span>
        <span className="step__title">{step.title}</span>
        <span className="step__intent">{INTENT_LABEL[step.intent]}</span>
        <span className="step__status">{step.status.replace("_", " ")}</span>
      </div>

      <div className="step__body">
        <p className="step__what">{step.what}</p>

        {step.how && (
          <details className="step__details">
            <summary>How</summary>
            <p>{step.how}</p>
          </details>
        )}

        {step.risk && (
          <details className="step__details step__details--risk">
            <summary>⚠ Risk</summary>
            <p>{step.risk}</p>
          </details>
        )}

        {step.backout && (
          <details className="step__details step__details--backout">
            <summary>↩ Backout</summary>
            <p>{step.backout}</p>
          </details>
        )}

        {step.retrospective && (
          <div className="retrospective">
            <h4>Retrospective ({step.retrospective.matches_plan})</h4>
            <p>{step.retrospective.done}</p>
            {step.retrospective.deviation && (
              <p className="retrospective__deviation">
                <strong>Deviation:</strong> {step.retrospective.deviation}
              </p>
            )}
          </div>
        )}
      </div>

      {step.status === "pending" && (
        <div className="step__actions">
          <button
            className="btn btn--sm btn--primary"
            onClick={() => onDelta({ type: "approve_step" } satisfies DeltaAction)}
          >
            ✓ Approve
          </button>
          <button
            className="btn btn--sm"
            onClick={() => {
              const reason = prompt("Why reject this step?");
              if (!reason) return;
              onDelta({ type: "reject_step", reason } satisfies DeltaAction);
            }}
          >
            ✗ Reject
          </button>
          <button
            className="btn btn--sm"
            onClick={() => {
              const reason = prompt("Why skip?");
              if (!reason) return;
              onDelta({ type: "skip_step", reason } satisfies DeltaAction);
            }}
          >
            ⤳ Skip
          </button>
          {editing ? (
            <StepEditor
              step={step}
              onSave={(action) => {
                onDelta(action);
                setEditing(false);
              }}
              onCancel={() => setEditing(false)}
            />
          ) : (
            <button className="btn btn--sm" onClick={startEditing}>
              ✎ Edit
            </button>
          )}
        </div>
      )}

      {step.status === "in_progress" && (
        <div className="step__actions">
          <button
            className="btn btn--sm btn--primary"
            onClick={() => {
              const actual = prompt("What was actually done?");
              if (!actual) return;
              onDelta({
                type: "complete_step",
                actual,
                matches_plan: "match",
                deviation: null,
              } satisfies DeltaAction);
            }}
          >
            ✓ Complete
          </button>
          <button
            className="btn btn--sm btn--danger"
            onClick={() => {
              const error = prompt("What went wrong?");
              if (!error) return;
              onDelta({ type: "fail_step", error } satisfies DeltaAction);
            }}
          >
            ✗ Fail
          </button>
        </div>
      )}
    </li>
  );
}

interface EditorProps {
  step: Step;
  onSave: (action: DeltaAction) => void;
  onCancel: () => void;
}

function StepEditor({ step, onSave, onCancel }: EditorProps) {
  const [title, setTitle] = useState(step.title);
  const [what, setWhat] = useState(step.what);

  return (
    <div className="step__editor">
      <label>
        title
        <input value={title} onChange={(e) => setTitle(e.target.value)} />
      </label>
      <label>
        what
        <textarea value={what} onChange={(e) => setWhat(e.target.value)} />
      </label>
      <div className="step__editor-actions">
        <button
          className="btn btn--sm btn--primary"
          onClick={() => {
            onSave({ type: "set_step_title", text: title } satisfies DeltaAction);
            onSave({ type: "set_step_what", text: what } satisfies DeltaAction);
          }}
        >
          Apply
        </button>
        <button className="btn btn--sm" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}
