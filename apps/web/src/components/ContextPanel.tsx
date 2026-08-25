import { Cpu, Lock, Unlock, Lightbulb, ListChecks, Pin } from "lucide-react";
import type { ContextSnapshot, ContextLayer as ContextLayerT, LayerKind } from "@/types/context";
import type { WorkingMemory } from "@/types/memory";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";

interface Props {
  snapshot: ContextSnapshot;
  workingMemory: WorkingMemory;
}

const LAYER_LABEL: Record<LayerKind, string> = {
  world_memory: "World memory",
  project_memory: "Project memory",
  session_memory: "Session memory",
  working_memory: "Working memory",
  step_memory: "Step memory",
};

export function ContextPanel({ snapshot, workingMemory }: Props) {
  const cacheable = snapshot.layers.filter((l) => l.is_cacheable);
  const variable = snapshot.layers.filter((l) => !l.is_cacheable);
  const cacheableTokens = cacheable.reduce((s, l) => s + l.tokens, 0);
  const variableTokens = variable.reduce((s, l) => s + l.tokens, 0);

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-sm">
            <Cpu className="h-4 w-4 text-primary" /> Context
          </CardTitle>
          <span className="font-mono text-[10px] text-muted-foreground">
            {snapshot.model.provider} / {snapshot.model.model}
          </span>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Cache boundary bar */}
        <div className="space-y-2">
          <div className="flex h-2 overflow-hidden rounded-full bg-secondary">
            <div
              className="bg-primary transition-all"
              style={{ flexGrow: cacheableTokens }}
              title={`${cacheableTokens} cacheable tokens`}
            />
            <div
              className="bg-amber-500 transition-all"
              style={{ flexGrow: variableTokens }}
              title={`${variableTokens} variable tokens`}
            />
          </div>
          <div className="flex items-center justify-between font-mono text-[10px] text-muted-foreground">
            <div className="flex items-center gap-3">
              <span className="flex items-center gap-1">
                <i className="inline-block h-2 w-2 rounded-sm bg-primary" />
                cacheable
              </span>
              <span className="flex items-center gap-1">
                <i className="inline-block h-2 w-2 rounded-sm bg-amber-500" />
                variable
              </span>
            </div>
            <span>
              {snapshot.total_input_tokens.toLocaleString()} /{" "}
              {snapshot.max_context_window.toLocaleString()}
            </span>
          </div>
        </div>

        <Separator />

        {/* Layers */}
        <div className="space-y-2.5">
          {snapshot.layers.map((l) => (
            <ContextLayerRow key={l.layer} layer={l} />
          ))}
        </div>

        <Separator />

        {/* Working memory */}
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="flex items-center gap-1.5 font-mono text-[10px] font-semibold uppercase tracking-wider text-amber-400">
              <Unlock className="h-3 w-3" /> Working memory
              <span className="text-muted-foreground">(variable only)</span>
            </h3>
          </div>

          <div className="grid grid-cols-2 gap-1.5 font-mono text-[10px] text-muted-foreground">
            <Kv k="plan" v={workingMemory.current_plan_id?.slice(0, 8) ?? "—"} />
            <Kv k="step" v={workingMemory.current_step_id?.slice(0, 8) ?? "—"} />
          </div>

          {workingMemory.hypotheses.length > 0 && (
            <WMSection icon={Lightbulb} title="hypotheses">
              <ul className="space-y-1">
                {workingMemory.hypotheses.map((h) => (
                  <li
                    key={h.id}
                    className="flex items-start gap-2 rounded-md border border-border/60 bg-background/30 px-2 py-1.5 text-xs"
                    data-status={h.status}
                  >
                    <Badge
                      variant={
                        h.status === "active"
                          ? "warning"
                          : h.status === "confirmed"
                            ? "success"
                            : h.status === "rejected"
                              ? "destructive"
                              : "outline"
                      }
                      className="shrink-0 font-mono text-[10px]"
                    >
                      {Math.round(h.confidence * 100)}%
                    </Badge>
                    <span className="text-foreground/90">{h.text}</span>
                  </li>
                ))}
              </ul>
            </WMSection>
          )}

          {workingMemory.decisions.length > 0 && (
            <WMSection icon={ListChecks} title="decisions">
              <ul className="space-y-1.5">
                {workingMemory.decisions.map((d) => (
                  <li
                    key={d.id}
                    className="rounded-md border border-border/60 bg-background/30 px-2 py-1.5 text-xs"
                  >
                    <strong className="font-semibold text-foreground">{d.text}</strong>
                    <span className="text-muted-foreground"> — {d.reason}</span>
                  </li>
                ))}
              </ul>
            </WMSection>
          )}

          {workingMemory.open_questions.length > 0 && (
            <WMSection icon={HelpCircle2} title="open questions">
              <ul className="space-y-1">
                {workingMemory.open_questions.map((q) => (
                  <li
                    key={q.id}
                    className="flex items-start gap-2 rounded-md border border-border/60 bg-background/30 px-2 py-1.5 text-xs"
                  >
                    {q.blocking && (
                      <Badge variant="danger" className="shrink-0 text-[10px]">
                        blocking
                      </Badge>
                    )}
                    <span className="text-foreground/90">{q.text}</span>
                  </li>
                ))}
              </ul>
            </WMSection>
          )}

          {workingMemory.pinned_facts.length > 0 && (
            <WMSection icon={Pin} title="pinned">
              <ul className="space-y-1">
                {workingMemory.pinned_facts.map((f) => (
                  <li
                    key={f.id}
                    className="rounded-md border border-border/60 bg-background/30 px-2 py-1.5 text-xs"
                  >
                    <span className="text-foreground/90">{f.text}</span>
                    <span className="ml-1 text-muted-foreground">— {f.source}</span>
                  </li>
                ))}
              </ul>
            </WMSection>
          )}
        </div>

        <Separator />

        {/* Cache stats */}
        <div className="grid grid-cols-3 gap-2 font-mono text-[10px]">
          <Stat
            label="prefix"
            value={`${snapshot.cache_boundary.prefix_tokens.toLocaleString()}`}
            icon={Lock}
          />
          <Stat
            label="hash"
            value={snapshot.cache_boundary.prefix_hash}
            icon={Cpu}
          />
          <Stat
            label="hits"
            value={String(snapshot.cache_boundary.cache_hits_estimated)}
            icon={Lock}
          />
        </div>
      </CardContent>
    </Card>
  );
}

function ContextLayerRow({ layer }: { layer: ContextLayerT }) {
  const pct = Math.min(100, Math.round((layer.tokens / layer.token_budget) * 100));
  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between font-mono text-[10px]">
        <span
          className={
            layer.is_cacheable
              ? "flex items-center gap-1 text-foreground/85"
              : "flex items-center gap-1 text-amber-400"
          }
        >
          {layer.is_cacheable ? (
            <Lock className="h-2.5 w-2.5" />
          ) : (
            <Unlock className="h-2.5 w-2.5" />
          )}
          {LAYER_LABEL[layer.layer]}
        </span>
        <span className="text-muted-foreground font-tabular">
          {layer.tokens.toLocaleString()} / {layer.token_budget.toLocaleString()}
        </span>
      </div>
      <Progress
        value={pct}
        className={`h-1.5 ${layer.is_cacheable ? "" : "[&>div]:bg-amber-500"}`}
      />
    </div>
  );
}

function WMSection({
  icon: Icon,
  title,
  children,
}: {
  icon: React.ComponentType<{ className?: string }>;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-1.5">
      <h4 className="flex items-center gap-1.5 font-mono text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        <Icon className="h-3 w-3" /> {title}
      </h4>
      {children}
    </div>
  );
}

function Kv({ k, v }: { k: string; v: string }) {
  return (
    <div className="flex items-center gap-1.5 rounded border border-border/60 bg-background/30 px-1.5 py-1">
      <span className="text-muted-foreground">{k}:</span>
      <span className="text-foreground/85">{v}</span>
    </div>
  );
}

function Stat({
  label,
  value,
  icon: Icon,
}: {
  label: string;
  value: string;
  icon: React.ComponentType<{ className?: string }>;
}) {
  return (
    <div className="rounded border border-border/60 bg-background/30 px-2 py-1.5">
      <div className="flex items-center gap-1 text-muted-foreground">
        <Icon className="h-2.5 w-2.5" />
        <span>{label}</span>
      </div>
      <div className="mt-0.5 truncate text-foreground/85">{value}</div>
    </div>
  );
}

// Re-import to avoid unused warning
import { HelpCircle as HelpCircle2 } from "lucide-react";
