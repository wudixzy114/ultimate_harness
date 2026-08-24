import type { PubItem } from "./types";

interface Props {
  item: PubItem;
}

// Renders an enum's variants as a flat table.
// `v0.1` limitation: variant `doc` extraction isn't in the lib yet —
// the doc column shows `—` for every row. v0.2 will extract per-variant
// `///` comments so the same undoc-highlight pattern as FieldTable applies.
export function VariantTable({ item }: Props) {
  const variants = (item.children ?? []).filter((c) => c.kind === "const");
  if (variants.length === 0) {
    return <div className="review-variants review-variants--empty">no variants</div>;
  }
  return (
    <div className="review-variants">
      <div className="review-variants__head">
        <span>variants</span>
        <span className="review-variants__count">{variants.length}</span>
      </div>
      <table className="review-variants__table">
        <thead>
          <tr>
            <th>name</th>
            <th>doc</th>
            <th>line</th>
          </tr>
        </thead>
        <tbody>
          {variants.map((v) => (
            <tr key={v.name}>
              <td className="review-variants__name">
                <code>{v.name}</code>
              </td>
              <td className="review-variants__doc">—</td>
              <td className="review-variants__line">{v.line}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
