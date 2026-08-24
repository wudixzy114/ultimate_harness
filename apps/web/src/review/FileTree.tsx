import type { FileSummary } from "./types";
import { fileGroup } from "./api";

interface Props {
  files: FileSummary[];
  selected: string | null;
  onSelect: (path: string) => void;
}

export function FileTree({ files, selected, onSelect }: Props) {
  const groups = new Map<string, FileSummary[]>();
  for (const f of files) {
    const g = fileGroup(f.path);
    if (!groups.has(g)) groups.set(g, []);
    groups.get(g)!.push(f);
  }
  const sortedGroups = [...groups.entries()].sort(([a], [b]) => a.localeCompare(b));

  return (
    <div className="review-tree">
      {sortedGroups.map(([group, items]) => (
        <div key={group} className="review-tree__group">
          <div className="review-tree__group-name">{group}</div>
          {items
            .sort((a, b) => a.path.localeCompare(b.path))
            .map((f) => {
              const isActive = selected === f.path;
              const short = f.path.replace(/^[^/\\]+[\\/]/, "");
              return (
                <button
                  key={f.path}
                  className={
                    "review-tree__item" + (isActive ? " review-tree__item--active" : "")
                  }
                  onClick={() => onSelect(f.path)}
                  title={f.purpose || "(no purpose)"}
                >
                  <span className="review-tree__item-name">{short}</span>
                  <span className="review-tree__item-meta">
                    {f.exposed_count}
                    {f.test_count > 0 ? ` · ${f.test_count}t` : ""}
                  </span>
                </button>
              );
            })}
        </div>
      ))}
    </div>
  );
}
