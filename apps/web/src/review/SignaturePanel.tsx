import type { PubItem } from "./types";

interface Props {
  item: PubItem;
}

// Renders a function's *body* (signature). In v0.1 the lib doesn't
// extract full param/return signatures, so the panel is intentionally
// a small placeholder showing the fn name. The DataFlow / invariants
// sections are rendered by DetailPanel uniformly across all kinds.
export function SignaturePanel({ item }: Props) {
  return (
    <div className="space-y-2">
      <h3 className="font-mono text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        signature
      </h3>
      <div className="rounded-md border border-sky-500/20 bg-sky-500/5 px-3 py-3">
        <em className="block text-xs text-muted-foreground">
          full signature extraction in v0.2
        </em>
        <code className="mt-2 block font-mono text-sm text-sky-400">
          fn {item.name.split(/::/).pop() ?? item.name}(...)
        </code>
      </div>
    </div>
  );
}
