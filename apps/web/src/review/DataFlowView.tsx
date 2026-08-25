import type { DataFlow } from "./types";
import { ArrowDownToLine, ArrowUpFromLine, Zap, Link2 } from "lucide-react";

// Renders a DataFlow as a small grid: input / output / side-effect / depends-on.
export function DataFlowView({ df }: { df: DataFlow }) {
  return (
    <dl className="grid gap-1.5 text-sm">
      {df.input && (
        <Row icon={ArrowDownToLine} k="input">
          <code className="text-cyan-400">{df.input}</code>
        </Row>
      )}
      {df.output && (
        <Row icon={ArrowUpFromLine} k="output">
          <code className="text-emerald-400">{df.output}</code>
        </Row>
      )}
      {df.side_effects && df.side_effects.length > 0 && (
        <Row icon={Zap} k="side-effect">
          <ul className="space-y-0.5">
            {df.side_effects.map((s, i) => (
              <li key={i} className="text-amber-400">
                {s}
              </li>
            ))}
          </ul>
        </Row>
      )}
      {df.depends_on && df.depends_on.length > 0 && (
        <Row icon={Link2} k="depends-on">
          <ul className="space-y-0.5">
            {df.depends_on.map((s, i) => (
              <li key={i} className="text-purple-400">
                {s}
              </li>
            ))}
          </ul>
        </Row>
      )}
    </dl>
  );
}

function Row({
  icon: Icon,
  k,
  children,
}: {
  icon: React.ComponentType<{ className?: string }>;
  k: string;
  children: React.ReactNode;
}) {
  return (
    <div className="grid grid-cols-[80px_1fr] items-start gap-2">
      <dt className="flex items-center gap-1.5 font-mono text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        <Icon className="h-3 w-3" />
        {k}
      </dt>
      <dd>{children}</dd>
    </div>
  );
}
