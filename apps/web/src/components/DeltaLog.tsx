import { Inbox, Workflow, ArrowRight } from "lucide-react";
import type { Delta } from "@/types/delta";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";

interface Props {
  deltas: Delta[];
}

export function DeltaLog({ deltas }: Props) {
  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-sm">
            <Workflow className="h-4 w-4 text-primary" /> Delta log
          </CardTitle>
          <Badge variant="default" className="font-mono">
            {deltas.length}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-xs text-muted-foreground">
          Every user interaction is captured as a structured{" "}
          <code className="rounded bg-muted px-1 py-0.5 font-mono text-[10px]">Delta</code>{" "}
          — the LLM never re-reads the plan; it only sees the delta.
        </p>

        {deltas.length === 0 ? (
          <div className="flex flex-col items-center gap-2 rounded-lg border border-dashed border-border/60 bg-background/30 py-8 text-center">
            <Inbox className="h-6 w-6 text-muted-foreground" />
            <p className="text-xs text-muted-foreground">
              No deltas yet. Click ✓ Approve / ✗ Reject / ✎ Edit on a step,
              <br />
              or use the plan actions below.
            </p>
          </div>
        ) : (
          <ScrollArea className="h-64 rounded-md border border-border/60 bg-background/30">
            <ol className="space-y-1 p-2 font-mono text-[11px]">
              {deltas
                .slice()
                .reverse()
                .map((d, i) => (
                  <li
                    key={d.id}
                    className="flex items-center gap-2 rounded border border-border/40 bg-card/40 px-2 py-1.5"
                  >
                    <Badge
                      variant="outline"
                      className="shrink-0 font-mono text-[10px] text-muted-foreground"
                    >
                      #{deltas.length - i}
                    </Badge>
                    <Badge
                      variant={"plan" in d.target ? "secondary" : "outline"}
                      className="shrink-0 font-mono text-[10px]"
                    >
                      {"plan" in d.target ? "plan" : "step"}
                    </Badge>
                    <span className="flex-1 truncate text-foreground/85">
                      {actionLabel(d)}
                    </span>
                    <span className="shrink-0 text-[10px] text-muted-foreground font-tabular">
                      {new Date(d.timestamp).toLocaleTimeString()}
                    </span>
                  </li>
                ))}
            </ol>
          </ScrollArea>
        )}

        {deltas.length > 0 && (
          <div className="rounded-md border border-primary/30 bg-primary/5 p-2.5 text-[11px] text-foreground/85">
            <p className="mb-1 flex items-center gap-1.5 font-mono text-[10px] font-semibold uppercase tracking-wider text-primary">
              <ArrowRight className="h-3 w-3" /> Token economics
            </p>
            {deltas.length} delta{deltas.length === 1 ? "" : "s"} emitted ≈
            {" "}
            <strong className="text-foreground">
              {deltas.length * 50}
            </strong>{" "}
            tokens sent to the LLM, vs ~{(deltas.length * 350) + 300} tokens for the
            equivalent text-based iteration.
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function actionLabel(d: Delta): string {
  if ("plan" in d.target) return `plan · ${d.action.type}`;
  return `step · ${d.action.type}`;
}
