import type { PubItem, TestInfo, FnAnalysis } from "./types";
import { DataFlowView } from "./DataFlowView";
import { FilePanel } from "./FilePanel";
import { FieldTable } from "./FieldTable";
import { VariantTable } from "./VariantTable";
import { MethodList } from "./MethodList";
import { SignaturePanel } from "./SignaturePanel";
import { DocPanel } from "./DocPanel";
import { Section, Divider } from "./common";
import { KindBadge } from "./ExposedTree";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { TagList } from "./common";
import { Inbox } from "lucide-react";

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
      <div className="flex h-full flex-col items-center justify-center gap-2 px-4 text-center font-mono text-xs text-muted-foreground">
        <Inbox className="h-6 w-6 opacity-50" />
        <p>select a file or item</p>
      </div>
    );
  }
  if (subject.kind === "file") {
    return (
      <ScrollArea className="h-full">
        <div className="p-5">
          <FilePanel
            purpose={subject.purpose}
            file_tags={subject.file_tags}
            file_invariants={subject.file_invariants}
            tests={subject.tests}
            fns={subject.fns}
          />
        </div>
      </ScrollArea>
    );
  }
  const it = subject.item;
  return (
    <ScrollArea className="h-full">
      <div className="space-y-4 p-5">
        <ItemHeader item={it} />
        <Divider />
        <KindPanel item={it} onSelectNested={subject.onSelectNested} />
        {it.invariants && it.invariants.length > 0 && (
          <>
            <Divider />
            <Section title="invariants">
              <ul className="space-y-1">
                {it.invariants.map((s, i) => (
                  <li
                    key={i}
                    className="rounded-md border border-amber-500/20 bg-amber-500/5 px-3 py-2 text-sm leading-relaxed text-foreground/90"
                  >
                    {s}
                  </li>
                ))}
              </ul>
            </Section>
          </>
        )}
        {it.data_flow && (
          <>
            <Divider />
            <Section title="data-flow">
              <DataFlowView df={it.data_flow} />
            </Section>
          </>
        )}
      </div>
    </ScrollArea>
  );
}

function ItemHeader({ item }: { item: PubItem }) {
  return (
    <header className="space-y-2">
      <div className="flex flex-wrap items-center gap-2">
        <KindBadge kind={item.kind} />
        <code className="text-base font-semibold text-foreground">
          {item.name}
        </code>
        <Badge variant="outline" className="font-mono text-[10px]">
          :{item.line}
        </Badge>
      </div>
      {item.tags && item.tags.length > 0 && <TagList tags={item.tags} />}
    </header>
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
