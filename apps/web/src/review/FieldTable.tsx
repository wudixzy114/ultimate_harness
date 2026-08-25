import type { FieldInfo, PubItem } from "./types";
import { cn } from "@/lib/utils";
import { AlertTriangle } from "lucide-react";

interface Props {
  item: PubItem;
  selectedField: string | null;
  onSelectField: (name: string) => void;
}

export function FieldTable({ item, selectedField, onSelectField }: Props) {
  if (item.fields === undefined || item.fields.length === 0) {
    return (
      <div className="rounded-md border border-dashed border-border/60 bg-background/30 px-3 py-6 text-center font-mono text-xs text-muted-foreground">
        {item.kind === "struct" ? "no fields" : `no fields (kind: ${item.kind})`}
      </div>
    );
  }
  const undocCount = item.fields.filter((f) => !f.has_doc).length;

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <h3 className="font-mono text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
          fields
        </h3>
        <span className="rounded bg-secondary px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">
          {item.fields.length}
        </span>
        {undocCount > 0 && (
          <span className="inline-flex items-center gap-1 rounded bg-red-500/15 px-1.5 py-0.5 font-mono text-[10px] text-red-400">
            <AlertTriangle className="h-2.5 w-2.5" />
            {undocCount} undoc
          </span>
        )}
      </div>
      <div className="overflow-hidden rounded-md border border-border/60">
        <table className="w-full text-sm">
          <thead className="bg-background/50">
            <tr className="border-b border-border/60 text-left font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
              <th className="px-3 py-2">name</th>
              <th className="px-3 py-2">type</th>
              <th className="px-3 py-2">doc</th>
              <th className="px-3 py-2 text-right">line</th>
            </tr>
          </thead>
          <tbody>
            {item.fields.map((f) => (
              <FieldRow
                key={f.name}
                f={f}
                active={selectedField === f.name}
                onClick={() => onSelectField(f.name)}
              />
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function FieldRow({
  f,
  active,
  onClick,
}: {
  f: FieldInfo;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <tr
      onClick={onClick}
      className={cn(
        "cursor-pointer border-b border-border/30 transition-colors last:border-b-0",
        active
          ? "bg-primary/10"
          : !f.has_doc
            ? "bg-red-950/30 hover:bg-red-950/50"
            : "hover:bg-accent/40",
      )}
    >
      <td className="px-3 py-2">
        <div className="flex items-center gap-1.5">
          <span className="font-mono text-[10px] text-muted-foreground">
            {f.vis}
          </span>
          <code className="text-foreground">{f.name}</code>
          {!f.has_doc && (
            <span title="missing /// doc">
              <AlertTriangle className="h-3 w-3 text-red-400" />
            </span>
          )}
        </div>
      </td>
      <td className="px-3 py-2">
        <code className="text-cyan-400">{f.ty}</code>
      </td>
      <td className="px-3 py-2 text-foreground/85">
        {f.has_doc ? (
          f.doc
        ) : (
          <span className="font-mono text-xs text-muted-foreground">—</span>
        )}
      </td>
      <td className="px-3 py-2 text-right font-mono text-[10px] text-muted-foreground tabular-nums">
        {f.line}
      </td>
    </tr>
  );
}
