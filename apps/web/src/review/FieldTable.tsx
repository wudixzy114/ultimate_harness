import type { FieldInfo, PubItem } from "./types";

interface Props {
  item: PubItem;
  selectedField: string | null;
  onSelectField: (name: string) => void;
}

export function FieldTable({ item, selectedField, onSelectField }: Props) {
  if (item.fields === undefined || item.fields.length === 0) {
    return (
      <div className="review-fields review-fields--empty">
        {item.kind === "struct"
          ? "no fields"
          : `no fields (kind: ${item.kind})`}
      </div>
    );
  }
  const undocCount = item.fields.filter((f) => !f.has_doc).length;
  return (
    <div className="review-fields">
      <div className="review-fields__head">
        <span>fields</span>
        <span className="review-fields__count">
          {item.fields.length}
          {undocCount > 0 ? (
            <span className="review-fields__undoc"> · {undocCount} undoc</span>
          ) : null}
        </span>
      </div>
      <table className="review-fields__table">
        <thead>
          <tr>
            <th>name</th>
            <th>type</th>
            <th>doc</th>
            <th>line</th>
          </tr>
        </thead>
        <tbody>
          {item.fields.map((f) => (
            <FieldRow
              key={f.name}
              f={f}
              active={selectedField === f.name}
              onClick={() => onSelectField(f.name)}
            />
          ))}
        </tbody>
      </table>
    </div>
  );
}

function FieldRow({
  f,
  active,
  onClick,
}: {
  f: FieldInfo;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <tr
      onClick={onClick}
      className={
        "review-fields__row" +
        (active ? " review-fields__row--active" : "") +
        (!f.has_doc ? " review-fields__row--undoc" : "")
      }
    >
      <td className="review-fields__name">
        <span className="review-fields__vis">{f.vis}</span>
        <code>{f.name}</code>
        {!f.has_doc ? (
          <span className="review-fields__missing" title="missing /// doc">
            ⚠
          </span>
        ) : null}
      </td>
      <td>
        <code className="review-fields__ty">{f.ty}</code>
      </td>
      <td className="review-fields__doc">
        {f.has_doc ? f.doc : <span className="review-fields__doc-empty">—</span>}
      </td>
      <td className="review-fields__line">{f.line}</td>
    </tr>
  );
}
