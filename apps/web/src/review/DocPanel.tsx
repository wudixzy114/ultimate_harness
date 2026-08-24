import type { PubItem } from "./types";
import { ExposedTree } from "./ExposedTree";

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
    <div className="review-doc">
      {isMod ? (
        <div className="review-section">
          <div className="review-section__title">contents</div>
          {item.children && item.children.length > 0 ? (
            <ExposedTree
              items={item.children}
              selected={null}
              onSelect={onSelectNested ?? (() => {})}
            />
          ) : (
            <div className="review-doc__empty">no public items</div>
          )}
        </div>
      ) : null}
    </div>
  );
}
