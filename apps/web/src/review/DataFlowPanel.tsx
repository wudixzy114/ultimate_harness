import type { DataFlow, PubItem, TestInfo, FnAnalysis } from "./types";

type Subject =
  | { kind: "file"; purpose: string; file_tags?: string[]; file_invariants?: string[]; tests: TestInfo[]; fns: FnAnalysis[] }
  | { kind: "item"; item: PubItem };

export function DetailPanel({ subject }: { subject: Subject | null }) {
  if (!subject) {
    return <div className="review-detail review-detail--empty">select a file or item</div>;
  }
  if (subject.kind === "file") {
    return (
      <div className="review-detail">
        <Section title="file purpose">
          <p className="review-purpose">{subject.purpose || <em>(none)</em>}</p>
        </Section>
        {subject.file_tags && subject.file_tags.length > 0 ? (
          <Section title="file tags">
            <TagList tags={subject.file_tags} />
          </Section>
        ) : null}
        {subject.file_invariants && subject.file_invariants.length > 0 ? (
          <Section title="file invariants">
            <ul className="review-inv">
              {subject.file_invariants.map((s, i) => (
                <li key={i}>{s}</li>
              ))}
            </ul>
          </Section>
        ) : null}
        {subject.tests.length > 0 ? (
          <Section title={`tests (${subject.tests.length})`}>
            <ul className="review-tests">
              {subject.tests.map((t) => (
                <li key={t.name}>
                  <code>{t.name}</code>
                  <span className="review-line">:{t.line}</span>
                </li>
              ))}
            </ul>
          </Section>
        ) : null}
        {subject.fns.length > 0 ? (
          <Section title={`data-flow fns (${subject.fns.length})`}>
            {subject.fns.map((f) => (
              <DataFlowCard key={f.name} name={f.name} line={f.line} dataFlow={f.attrs.data_flow} />
            ))}
          </Section>
        ) : null}
      </div>
    );
  }
  // item
  const it = subject.item;
  return (
    <div className="review-detail">
      <Section title={`${it.kind} ${it.name}`}>
        <code className="review-line">:{it.line}</code>
        {it.tags && it.tags.length > 0 ? <TagList tags={it.tags} /> : null}
      </Section>
      {it.invariants && it.invariants.length > 0 ? (
        <Section title="invariants">
          <ul className="review-inv">
            {it.invariants.map((s, i) => (
              <li key={i}>{s}</li>
            ))}
          </ul>
        </Section>
      ) : null}
      {it.data_flow ? (
        <Section title="data-flow">
          <DataFlowView df={it.data_flow} />
        </Section>
      ) : null}
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="review-section">
      <div className="review-section__title">{title}</div>
      <div className="review-section__body">{children}</div>
    </div>
  );
}

function TagList({ tags }: { tags: string[] }) {
  return (
    <div className="review-tags">
      {tags.map((t) => (
        <span key={t} className="review-tag">
          {t}
        </span>
      ))}
    </div>
  );
}

function DataFlowCard({
  name,
  line,
  dataFlow,
}: {
  name: string;
  line: number;
  dataFlow?: DataFlow;
}) {
  if (!dataFlow) {
    return (
      <div className="review-df-card">
        <code>{name}</code>
        <span className="review-line">:{line}</span>
      </div>
    );
  }
  return (
    <div className="review-df-card">
      <div>
        <code>{name}</code>
        <span className="review-line">:{line}</span>
      </div>
      <DataFlowView df={dataFlow} />
    </div>
  );
}

function DataFlowView({ df }: { df: DataFlow }) {
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
