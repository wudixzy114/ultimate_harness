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
    <div className="review-sig">
      <div className="review-sig__placeholder">
        <em>full signature extraction in v0.2</em>
        <div className="review-sig__name">
          <code>fn {item.name.split(/::/).pop() ?? item.name}(…)</code>
        </div>
      </div>
    </div>
  );
}
