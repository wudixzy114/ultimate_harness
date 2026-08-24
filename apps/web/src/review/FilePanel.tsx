import type { TestInfo, FnAnalysis } from "./types";
import { DataFlowView } from "./DataFlowView";
import { Section, TagList } from "./common";

interface Props {
  purpose: string;
  file_tags?: string[];
  file_invariants?: string[];
  tests: TestInfo[];
  fns: FnAnalysis[];
}

// Renders the right-side panel for a file-level selection
// (no item picked yet). Shows purpose / tags / invariants / tests /
// file-level data-flow functions.
export function FilePanel({
  purpose,
  file_tags,
  file_invariants,
  tests,
  fns,
}: Props) {
  return (
    <div className="review-file">
      <Section title="file purpose">
        <p className="review-purpose">{purpose || <em>(none)</em>}</p>
      </Section>
      {file_tags && file_tags.length > 0 ? (
        <Section title="file tags">
          <TagList tags={file_tags} />
        </Section>
      ) : null}
      {file_invariants && file_invariants.length > 0 ? (
        <Section title="file invariants">
          <ul className="review-inv">
            {file_invariants.map((s, i) => (
              <li key={i}>{s}</li>
            ))}
          </ul>
        </Section>
      ) : null}
      {tests.length > 0 ? (
        <Section title={`tests (${tests.length})`}>
          <ul className="review-tests">
            {tests.map((t) => (
              <li key={t.name}>
                <code>{t.name}</code>
                <span className="review-line">:{t.line}</span>
              </li>
            ))}
          </ul>
        </Section>
      ) : null}
      {fns.length > 0 ? (
        <Section title={`data-flow fns (${fns.length})`}>
          {fns.map((f) => (
            <div key={f.name} className="review-df-card">
              <div>
                <code>{f.name}</code>
                <span className="review-line">:{f.line}</span>
              </div>
              {f.attrs.data_flow ? <DataFlowView df={f.attrs.data_flow} /> : null}
            </div>
          ))}
        </Section>
      ) : null}
    </div>
  );
}
