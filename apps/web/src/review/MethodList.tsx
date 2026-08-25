import type { PubItem } from "./types";

interface Props {
  item: PubItem;
}

// Renders a trait/impl's methods as a flat table.
// v0.1 limitation: full signatures (params, return type) aren't in
// the lib yet — the signature column shows the method name. v0.2 will
// add per-method signature extraction.
export function MethodList({ item }: Props) {
  const methods = (item.children ?? []).filter(
    (c) => c.kind === "fn" || c.kind === "impl",
  );
  if (methods.length === 0) {
    return (
      <div className="rounded-md border border-dashed border-border/60 bg-background/30 px-3 py-6 text-center font-mono text-xs text-muted-foreground">
        no methods
      </div>
    );
  }
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <h3 className="font-mono text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
          methods
        </h3>
        <span className="rounded bg-secondary px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">
          {methods.length}
        </span>
      </div>
      <div className="overflow-hidden rounded-md border border-border/60">
        <table className="w-full text-sm">
          <thead className="bg-background/50">
            <tr className="border-b border-border/60 text-left font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
              <th className="px-3 py-2">signature</th>
              <th className="px-3 py-2">doc</th>
              <th className="px-3 py-2 text-right">line</th>
            </tr>
          </thead>
          <tbody>
            {methods.map((m) => (
              <tr
                key={m.name}
                className="border-b border-border/30 last:border-b-0 hover:bg-accent/40"
              >
                <td className="px-3 py-2">
                  <code className="text-sky-400">{m.name}</code>
                </td>
                <td className="px-3 py-2 font-mono text-xs text-muted-foreground">
                  —
                </td>
                <td className="px-3 py-2 text-right font-mono text-[10px] text-muted-foreground tabular-nums">
                  {m.line}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
