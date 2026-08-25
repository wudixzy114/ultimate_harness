import type { PubItem } from "./types";

interface Props {
  item: PubItem;
}

// Renders an enum's variants as a flat table.
// v0.1 limitation: variant `doc` extraction isn't in the lib yet —
// the doc column shows `—` for every row. v0.2 will extract per-variant
// `///` comments so the same undoc-highlight pattern as FieldTable applies.
export function VariantTable({ item }: Props) {
  const variants = (item.children ?? []).filter((c) => c.kind === "const");
  if (variants.length === 0) {
    return (
      <div className="rounded-md border border-dashed border-border/60 bg-background/30 px-3 py-6 text-center font-mono text-xs text-muted-foreground">
        no variants
      </div>
    );
  }
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <h3 className="font-mono text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
          variants
        </h3>
        <span className="rounded bg-secondary px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">
          {variants.length}
        </span>
      </div>
      <div className="overflow-hidden rounded-md border border-border/60">
        <table className="w-full text-sm">
          <thead className="bg-background/50">
            <tr className="border-b border-border/60 text-left font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
              <th className="px-3 py-2">name</th>
              <th className="px-3 py-2">doc</th>
              <th className="px-3 py-2 text-right">line</th>
            </tr>
          </thead>
          <tbody>
            {variants.map((v) => (
              <tr
                key={v.name}
                className="border-b border-border/30 last:border-b-0 hover:bg-accent/40"
              >
                <td className="px-3 py-2">
                  <code className="text-amber-400">{v.name}</code>
                </td>
                <td className="px-3 py-2 font-mono text-xs text-muted-foreground">
                  —
                </td>
                <td className="px-3 py-2 text-right font-mono text-[10px] text-muted-foreground tabular-nums">
                  {v.line}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
