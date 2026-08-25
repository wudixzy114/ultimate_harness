import type { PubItem } from "./types";
import { ExposedTree } from "./ExposedTree";
import { Section } from "./common";
import { Inbox } from "lucide-react";

interface Props {
  item: PubItem;
  onSelectNested?: (name: string) => void;
}

// Body-only panel for `const` / `type` / `mod`.
// Header (kind/name/line/tags) is rendered uniformly by DetailPanel.
// For `mod` we additionally render a nested ExposedTree of the mod's
// own pub items; nested selection (if any) bubbles up via onSelectNested.
export function DocPanel({ item, onSelectNested }: Props) {
  const isMod = item.kind === "mod";
  return (
    <div className="space-y-2">
      {isMod ? (
        <Section title="contents">
          {item.children && item.children.length > 0 ? (
            <div className="rounded-md border border-border/60 bg-background/30">
              <ExposedTree
                items={item.children}
                selected={null}
                onSelect={onSelectNested ?? (() => {})}
              />
            </div>
          ) : (
            <div className="flex flex-col items-center gap-1 rounded-md border border-dashed border-border/60 bg-background/30 py-6 text-center font-mono text-xs text-muted-foreground">
              <Inbox className="h-4 w-4 opacity-50" />
              no public items
            </div>
          )}
        </Section>
      ) : (
        <div className="rounded-md border border-dashed border-border/60 bg-background/30 px-3 py-6 text-center font-mono text-xs text-muted-foreground">
          (no body extraction yet for {item.kind})
        </div>
      )}
    </div>
  );
}
