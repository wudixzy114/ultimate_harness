import { useEffect, useState } from "react";
import { Boxes, FileCode2, Cpu, Workflow } from "lucide-react";
import { PlanPanel } from "./components/PlanPanel";
import { ContextPanel } from "./components/ContextPanel";
import { DeltaLog } from "./components/DeltaLog";
import { SAMPLE_PLAN, SAMPLE_CONTEXT, SAMPLE_WORKING_MEMORY } from "./data/samplePlan";
import type { Delta } from "./types/delta";
import { Badge } from "./components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./components/ui/tabs";
import { TooltipProvider } from "./components/ui/tooltip";
import { Review } from "./review";
import { Button } from "./components/ui/button";
import { Code2 } from "lucide-react";

function isReviewHash() {
  return typeof window !== "undefined" && window.location.hash === "#/review";
}

export function App() {
  // Code viewer is a dev tool, routed via URL hash to keep the product
  // shell untouched. Visit `/#/review` to enter.
  const [isReview, setIsReview] = useState(isReviewHash);
  useEffect(() => {
    const handler = () => setIsReview(isReviewHash());
    window.addEventListener("hashchange", handler);
    return () => window.removeEventListener("hashchange", handler);
  }, []);

  if (isReview) {
    return (
      <div className="flex h-screen flex-col bg-background text-foreground">
        <Review />
      </div>
    );
  }

  const [deltas, setDeltas] = useState<Delta[]>([]);

  return (
    <TooltipProvider delayDuration={300}>
      <div className="flex h-screen flex-col bg-background text-foreground">
        <Header onOpenReview={() => (window.location.hash = "#/review")} />

        <main className="grid flex-1 grid-cols-[1fr_400px] overflow-hidden">
          <div className="overflow-y-auto bg-background/50">
            <PlanPanel plan={SAMPLE_PLAN} onDelta={(d) => setDeltas((prev) => [...prev, d])} />
          </div>
          <aside className="flex flex-col gap-4 overflow-y-auto border-l border-border bg-card/30 p-4">
            <Tabs defaultValue="context" className="flex flex-col gap-3">
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="context" className="gap-1.5">
                  <Cpu className="h-3.5 w-3.5" /> Context
                </TabsTrigger>
                <TabsTrigger value="deltas" className="gap-1.5">
                  <Workflow className="h-3.5 w-3.5" /> Delta log
                  {deltas.length > 0 && (
                    <Badge variant="default" className="ml-1 h-4 px-1.5 text-[10px]">
                      {deltas.length}
                    </Badge>
                  )}
                </TabsTrigger>
              </TabsList>
              <TabsContent value="context" className="mt-0">
                <ContextPanel
                  snapshot={SAMPLE_CONTEXT}
                  workingMemory={SAMPLE_WORKING_MEMORY}
                />
              </TabsContent>
              <TabsContent value="deltas" className="mt-0">
                <DeltaLog deltas={deltas} />
              </TabsContent>
            </Tabs>
          </aside>
        </main>

        <Footer />
      </div>
    </TooltipProvider>
  );
}

function Header({ onOpenReview }: { onOpenReview: () => void }) {
  return (
    <header className="flex items-center justify-between border-b border-border bg-card/40 px-6 py-3">
      <div className="flex items-center gap-3">
        <div className="flex h-8 w-8 items-center justify-center rounded-md bg-primary/10 text-primary ring-1 ring-primary/20">
          <Boxes className="h-4 w-4" />
        </div>
        <div>
          <h1 className="text-sm font-semibold leading-none tracking-tight">
            Ultimate <span className="text-primary">Harness</span>
          </h1>
          <p className="mt-0.5 font-mono text-[10px] text-muted-foreground">
            structured iteration is a first-class concern
          </p>
        </div>
      </div>
      <div className="flex items-center gap-2 font-mono text-[10px] text-muted-foreground">
        <Badge variant="outline" className="font-mono">v0.0.1</Badge>
        <span>·</span>
        <span>session {SAMPLE_PLAN.session_id.slice(0, 8)}</span>
        <Button
          size="sm"
          variant="outline"
          onClick={onOpenReview}
          className="ml-2 h-7 gap-1.5 px-2 text-[11px]"
          title="Open Rust code viewer (dev tool) at /#/review"
        >
          <Code2 className="h-3 w-3" /> /review
        </Button>
      </div>
    </header>
  );
}

function Footer() {
  return (
    <footer className="flex items-center gap-2 border-t border-border bg-card/40 px-6 py-1.5 font-mono text-[10px] text-muted-foreground">
      <FileCode2 className="h-3 w-3" />
      <span>
        v0.0.1 — sample data only. v0.0.3 will connect via WebSocket to the Rust daemon.
      </span>
    </footer>
  );
}
