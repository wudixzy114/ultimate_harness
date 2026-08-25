import type { PubItem, PubKind } from "./types";
import { cn } from "@/lib/utils";

interface Props {
  items: PubItem[];
  selected: string | null;
  onSelect: (name: string) => void;
}

export function ExposedTree({ items, selected, onSelect }: Props) {
  if (items.length === 0) {
    return (
      <div className="px-4 py-6 text-center font-mono text-xs text-muted-foreground">
        no public items
      </div>
    );
  }
  return (
    <ul className="space-y-0.5 p-2">
      {items.map((it) => (
        <Item
          key={it.name}
          item={it}
          depth={0}
          selected={selected}
          onSelect={onSelect}
        />
      ))}
    </ul>
  );
}

function Item({
  item,
  depth,
  selected,
  onSelect,
}: {
  item: PubItem;
  depth: number;
  selected: string | null;
  onSelect: (name: string) => void;
}) {
  const isActive = selected === item.name;
  // Tree depth rule: only `mod` may expand one level. Other kinds show
  // as a single row — its content lives in the right-side panel.
  const canExpand = item.kind === "mod" && (item.children?.length ?? 0) > 0;
  const flags = computeFlags(item);

  return (
    <li>
      <button
        onClick={() => onSelect(item.name)}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        className={cn(
          "flex w-full items-center gap-2 rounded-md py-1.5 pr-2 text-left text-xs transition-colors",
          isActive
            ? "bg-primary/15 text-primary ring-1 ring-primary/30"
            : "text-foreground/85 hover:bg-accent/40",
        )}
      >
        <KindBadge kind={item.kind} />
        <code className="flex-1 truncate font-mono">{item.name}</code>
        {flags.length > 0 && (
          <span className="flex shrink-0 items-center gap-1">
            {flags.map((f) => (
              <FlagBadge key={f} flag={f} />
            ))}
          </span>
        )}
        <span className="shrink-0 font-mono text-[10px] text-muted-foreground tabular-nums">
          :{item.line}
        </span>
      </button>
      {canExpand && (
        <ul className="space-y-0.5">
          {item.children!.map((c) => (
            <Item
              key={c.name}
              item={c}
              depth={depth + 1}
              selected={selected}
              onSelect={onSelect}
            />
          ))}
        </ul>
      )}
    </li>
  );
}

function computeFlags(item: PubItem): string[] {
  const flags: string[] = [];
  if (item.kind === "struct" && item.has_undocumented_fields) flags.push("undoc");
  if (item.data_flow) flags.push("df");
  if ((item.invariants?.length ?? 0) > 0) flags.push("inv");
  return flags;
}

// ── Kind/Flag colors per design doc ──────────────────────────────

const KIND_STYLE: Record<PubKind, string> = {
  struct: "bg-cyan-500/15 text-cyan-400 ring-cyan-500/30",
  enum: "bg-amber-500/15 text-amber-400 ring-amber-500/30",
  trait: "bg-purple-500/15 text-purple-400 ring-purple-500/30",
  fn: "bg-sky-500/15 text-sky-400 ring-sky-500/30",
  const: "bg-blue-500/15 text-blue-400 ring-blue-500/30",
  type: "bg-emerald-500/15 text-emerald-400 ring-emerald-500/30",
  impl: "bg-purple-500/15 text-purple-300 ring-purple-500/30",
  mod: "bg-zinc-500/15 text-zinc-400 ring-zinc-500/30",
};

export function KindBadge({ kind }: { kind: PubKind }) {
  return (
    <span
      className={cn(
        "inline-flex shrink-0 items-center gap-0.5 rounded px-1.5 py-0.5 font-mono text-[10px] font-medium ring-1 ring-inset",
        KIND_STYLE[kind],
      )}
    >
      {kind}
    </span>
  );
}

const FLAG_STYLE: Record<string, string> = {
  undoc: "bg-red-500/15 text-red-400",
  df: "bg-cyan-500/15 text-cyan-400",
  inv: "bg-amber-500/15 text-amber-400",
};

function FlagBadge({ flag }: { flag: string }) {
  return (
    <span
      className={cn(
        "rounded px-1.5 py-0.5 font-mono text-[10px] font-medium",
        FLAG_STYLE[flag] ?? "bg-secondary text-muted-foreground",
      )}
      title={
        flag === "undoc"
          ? "undocumented fields present"
          : flag === "df"
            ? "has data-flow annotations"
            : flag === "inv"
              ? "has invariants"
              : flag
      }
    >
      {flag}
    </span>
  );
}
