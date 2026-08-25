import type { FileSummary } from "./types";
import { fileGroup } from "./api";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import { Folder, FileCode2 } from "lucide-react";

interface Props {
  files: FileSummary[];
  selected: string | null;
  onSelect: (path: string) => void;
}

export function FileTree({ files, selected, onSelect }: Props) {
  // Group files by their top-level segment (crate name).
  const groups = new Map<string, FileSummary[]>();
  for (const f of files) {
    const g = fileGroup(f.path);
    if (!groups.has(g)) groups.set(g, []);
    groups.get(g)!.push(f);
  }
  const sortedGroups = [...groups.entries()].sort(([a], [b]) =>
    a.localeCompare(b),
  );

  return (
    <ScrollArea className="h-full">
      <div className="space-y-3 p-2">
        {sortedGroups.map(([group, items]) => (
          <div key={group} className="space-y-1">
            <div className="flex items-center gap-1.5 px-2 py-1 font-mono text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
              <Folder className="h-3 w-3" />
              {group}
            </div>
            {items
              .sort((a, b) => a.path.localeCompare(b.path))
              .map((f) => {
                const isActive = selected === f.path;
                // Strip the leading crate segment to make paths compact.
                const short = f.path.replace(/^[^/\\]+[\\/]/, "");
                return (
                  <button
                    key={f.path}
                    onClick={() => onSelect(f.path)}
                    title={f.purpose || "(no purpose)"}
                    className={cn(
                      "group flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs transition-colors",
                      isActive
                        ? "bg-primary/15 text-primary ring-1 ring-primary/30"
                        : "text-foreground/85 hover:bg-accent/40 hover:text-foreground",
                    )}
                  >
                    <FileCode2
                      className={cn(
                        "h-3.5 w-3.5 shrink-0",
                        isActive ? "text-primary" : "text-muted-foreground",
                      )}
                    />
                    <span className="flex-1 truncate font-mono">{short}</span>
                    <span className="flex shrink-0 items-center gap-1.5 font-mono text-[10px] text-muted-foreground">
                      <span className="rounded bg-secondary px-1.5 py-0.5">
                        {f.exposed_count}
                      </span>
                      {f.test_count > 0 && (
                        <span className="rounded bg-emerald-500/15 px-1.5 py-0.5 text-emerald-400">
                          {f.test_count}t
                        </span>
                      )}
                    </span>
                  </button>
                );
              })}
          </div>
        ))}
      </div>
    </ScrollArea>
  );
}
