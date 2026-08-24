import type { PubItem } from "./types";

interface Props {
  item: PubItem;
}

// Renders a trait/impl's methods as a flat table.
// `v0.1` limitation: full signatures (params, return type) aren't in
// the lib yet — the signature column shows the method name. v0.2 will
// add per-method signature extraction.
export function MethodList({ item }: Props) {
  const methods = (item.children ?? []).filter(
    (c) => c.kind === "fn" || c.kind === "impl",
  );
  if (methods.length === 0) {
    return <div className="review-methods review-methods--empty">no methods</div>;
  }
  return (
    <div className="review-methods">
      <div className="review-methods__head">
        <span>methods</span>
        <span className="review-methods__count">{methods.length}</span>
      </div>
      <table className="review-methods__table">
        <thead>
          <tr>
            <th>signature</th>
            <th>doc</th>
            <th>line</th>
          </tr>
        </thead>
        <tbody>
          {methods.map((m) => (
            <tr key={m.name}>
              <td className="review-methods__sig">
                <code>{m.name}</code>
              </td>
              <td className="review-methods__doc">—</td>
              <td className="review-methods__line">{m.line}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
