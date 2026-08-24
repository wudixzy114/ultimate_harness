import { useState } from "react";
import { PlanPanel } from "./components/PlanPanel";
import { ContextPanel } from "./components/ContextPanel";
import { DeltaLog } from "./components/DeltaLog";
import { SAMPLE_PLAN, SAMPLE_CONTEXT, SAMPLE_WORKING_MEMORY } from "./data/samplePlan";
import type { Delta } from "./types/delta";

export function App() {
  const [deltas, setDeltas] = useState<Delta[]>([]);

  return (
    <div className="app">
      <header className="app__header">
        <h1 className="app__title">
          Ultimate <span className="app__title-accent">Harness</span>
        </h1>
        <div className="app__tagline">
          structured iteration is a first-class concern · v0.0.1
        </div>
      </header>

      <main className="app__main">
        <div className="app__plan">
          <PlanPanel plan={SAMPLE_PLAN} onDelta={(d) => setDeltas((prev) => [...prev, d])} />
        </div>
        <div className="app__side">
          <ContextPanel snapshot={SAMPLE_CONTEXT} workingMemory={SAMPLE_WORKING_MEMORY} />
          <DeltaLog deltas={deltas} />
        </div>
      </main>

      <footer className="app__footer">
        <span>
          This is v0.0.1 — sample data only. v0.0.3 will connect via WebSocket to the Rust daemon.
        </span>
      </footer>
    </div>
  );
}
