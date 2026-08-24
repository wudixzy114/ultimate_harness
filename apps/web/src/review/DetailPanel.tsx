import type { PubItem, TestInfo, FnAnalysis } from "./types";
import { DataFlowView } from "./DataFlowView";
import { FilePanel } from "./FilePanel";
import { FieldTable } from "./FieldTable";
import { VariantTable } from "./VariantTable";
import { MethodList } from "./MethodList";
import { SignaturePanel } from "./SignaturePanel";
import { DocPanel } from "./DocPanel";
import { Section, TagList } from "./common";

type Subject =
  | {
      kind: "file";
      purpose: string;
      file_tags?: string[];
      file_invariants?: string[];
      tests: TestInfo[];
      fns: FnAnalysis[];
    }
  | { kind: "item"; item: PubItem; onSelectNested?: (name: string) => void };

export function DetailPanel({ subject }: { subject: Subject | null }) {
  if (!subject) {
    return (
      <div className="review-detail review-detail--empty">
        select a file or item
      </div>
    );
  }
  if (subject.kind === "file") {
    return (
      <FilePanel
        purpose={subject.purpose}
        file_tags={subject.file_tags}
        file_invariants={subject.file_invariants}
        tests={subject.tests}
        fns={subject.fns}
      />
    );
  }
  const it = subject.item;
  return (
    <div className="review-detail">
      <ItemHeader item={it} />
      <KindPanel item={it} onSelectNested={subject.onSelectNested} />
      {it.invariants && it.invariants.length > 0 ? (
        <Section title="invariants">
          <ul className="review-inv">
            {it.invariants.map((s, i) => (
              <li key={i}>{s}</li>
            ))}
          </ul>
        </Section>
      ) : null}
      {it.data_flow ? (
        <Section title="data-flow">
          <DataFlowView df={it.data_flow} />
        </Section>
      ) : null}
    </div>
  );
}

function ItemHeader({ item }: { item: PubItem }) {
  return (
    <div className="review-item-header">
      <span className={`review-exposed__kind review-exposed__kind--${item.kind}`}>
        {item.kind}
      </span>
      <code className="review-item-header__name">{item.name}</code>
      <span className="review-line">:{item.line}</span>
      {item.tags && item.tags.length > 0 ? <TagList tags={item.tags} /> : null}
    </div>
  );
}

function KindPanel({
  item,
  onSelectNested,
}: {
  item: PubItem;
  onSelectNested?: (name: string) => void;
}) {
  switch (item.kind) {
    case "struct":
      return <FieldTable item={item} selectedField={null} onSelectField={() => {}} />;
    case "enum":
      return <VariantTable item={item} />;
    case "trait":
    case "impl":
      return <MethodList item={item} />;
    case "fn":
      return <SignaturePanel item={item} />;
    case "const":
    case "type":
    case "mod":
      return <DocPanel item={item} onSelectNested={onSelectNested} />;
  }
}
