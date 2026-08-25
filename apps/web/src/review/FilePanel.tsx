import type { TestInfo, FnAnalysis } from "./types";
import { DataFlowView } from "./DataFlowView";
import { Section, TagList } from "./common";
import { FlaskConical, ListOrdered } from "lucide-react";

interface Props {
  purpose: string;
  file_tags?: string[];
  file_invariants?: string[];
  tests: TestInfo[];
  fns: FnAnalysis[];
}

export function FilePanel({
  purpose,
  file_tags,
  file_invariants,
  tests,
  fns,
}: Props) {
  return (
    <div className="space-y-4">
      <Section title="file purpose">
        <p className="rounded-md border border-border/60 bg-background/30 px-3 py-2 text-sm leading-relaxed text-foreground/90">
          {purpose || <em className="text-muted-foreground">(none)</em>}
        </p>
      </Section>

      {file_tags && file_tags.length > 0 && (
        <Section title="file tags">
          <TagList tags={file_tags} />
        </Section>
      )}

      {file_invariants && file_invariants.length > 0 && (
        <Section title="file invariants">
          <ul className="space-y-1">
            {file_invariants.map((s, i) => (
              <li
                key={i}
                className="rounded-md border border-amber-500/20 bg-amber-500/5 px-3 py-2 text-sm leading-relaxed text-foreground/90"
              >
                {s}
              </li>
            ))}
          </ul>
        </Section>
      )}

      {tests.length > 0 && (
        <Section
          title="tests"
          count={
            <span className="rounded bg-emerald-500/15 px-1.5 py-0.5 font-mono text-[10px] text-emerald-400">
              {tests.length}
            </span>
          }
        >
          <ul className="space-y-1">
            {tests.map((t) => (
              <li
                key={t.name}
                className="flex items-center gap-2 rounded-md border border-border/60 bg-background/30 px-3 py-1.5 text-sm"
              >
                <FlaskConical className="h-3.5 w-3.5 text-emerald-400" />
                <code className="flex-1 truncate font-mono">{t.name}</code>
                <span className="font-mono text-[10px] text-muted-foreground tabular-nums">
                  :{t.line}
                </span>
              </li>
            ))}
          </ul>
        </Section>
      )}

      {fns.length > 0 && (
        <Section
          title="data-flow fns"
          count={
            <span className="rounded bg-cyan-500/15 px-1.5 py-0.5 font-mono text-[10px] text-cyan-400">
              {fns.length}
            </span>
          }
        >
          <div className="space-y-2">
            {fns.map((f) => (
              <div
                key={f.name}
                className="rounded-md border border-cyan-500/20 bg-cyan-500/5 p-3"
              >
                <div className="mb-2 flex items-center gap-2">
                  <ListOrdered className="h-3.5 w-3.5 text-cyan-400" />
                  <code className="font-mono text-sm">{f.name}</code>
                  <span className="font-mono text-[10px] text-muted-foreground tabular-nums">
                    :{f.line}
                  </span>
                </div>
                {f.attrs.data_flow && <DataFlowView df={f.attrs.data_flow} />}
              </div>
            ))}
          </div>
        </Section>
      )}
    </div>
  );
}
