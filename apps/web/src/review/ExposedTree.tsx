import type { PubItem } from "./types";

interface Props {
  items: PubItem[];
  selected: string | null;
  onSelect: (name: string) => void;
}

export function ExposedTree({ items, selected, onSelect }: Props) {
  if (items.length === 0) {
    return <div className="review-exposed review-exposed--empty">no public items</div>;
  }
  return (
    <ul className="review-exposed">
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
  // Tree depth rule: only `mod` may expand one level. Every other kind
  // shows as a single row — its content lives in the right-side panel.
  const canExpand =
    item.kind === "mod" && (item.children?.length ?? 0) > 0;
  const flags = computeFlags(item);
  return (
    <li>
      <button
        className={
          "review-exposed__item" + (isActive ? " review-exposed__item--active" : "")
        }
        onClick={() => onSelect(item.name)}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
      >
        <span className={`review-exposed__kind review-exposed__kind--${item.kind}`}>
          {item.kind}
        </span>
        <code className="review-exposed__name">{item.name}</code>
        {flags.length > 0 ? (
          <span className="review-exposed__flags">
            {flags.map((f) => (
              <span
                key={f}
                className={`review-exposed__flag review-exposed__flag--${f}`}
              >
                {f}
              </span>
            ))}
          </span>
        ) : null}
        <span className="review-exposed__line">:{item.line}</span>
      </button>
      {canExpand ? (
        <ul className="review-exposed">
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
      ) : null}
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
