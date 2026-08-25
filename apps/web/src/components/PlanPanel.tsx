import { useState } from "react";
import {
  Check,
  X,
  SkipForward,
  Pencil,
  CircleDashed,
  Search,
  Wrench,
  CheckCheck,
  HelpCircle,
  Eye,
  MessageSquare,
  ChevronDown,
} from "lucide-react";
import type { Plan, Step, StepIntent, StepStatus } from "@/types/plan";
import type { Delta, DeltaAction } from "@/types/delta";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogClose,
} from "@/components/ui/dialog";

const INTENT_ICON: Record<StepIntent, React.ComponentType<{ className?: string }>> = {
  investigate: Search,
  implement: Wrench,
  verify: CheckCheck,
  ask_user: HelpCircle,
  review: Eye,
  communicate: MessageSquare,
};

const INTENT_LABEL: Record<StepIntent, string> = {
  investigate: "Investigate",
  implement: "Implement",
  verify: "Verify",
  ask_user: "Ask User",
  review: "Review",
  communicate: "Communicate",
};

const STATUS_VARIANT: Record<
  StepStatus,
  "outline" | "warning" | "success" | "destructive" | "secondary" | "default"
> = {
  pending: "outline",
  in_progress: "warning",
  done: "success",
  failed: "destructive",
  skipped: "secondary",
  blocked: "destructive",
};

const STATUS_DOT: Record<StepStatus, string> = {
  pending: "bg-status-pending",
  in_progress: "bg-status-progress",
  done: "bg-status-done",
  failed: "bg-status-failed",
  skipped: "bg-status-skipped",
  blocked: "bg-status-blocked",
};

interface Props {
  plan: Plan;
  onDelta?: (delta: Delta) => void;
}

export function PlanPanel({ plan, onDelta }: Props) {
  const verifiedCount = plan.success_criteria.filter((c) => c.verified).length;
  const totalCriteria = plan.success_criteria.length;

  return (
    <div className="mx-auto max-w-3xl space-y-6 p-8">
      {/* Header card */}
      <Card>
        <CardHeader>
          <div className="flex items-start justify-between gap-3">
            <div className="space-y-1.5">
              <CardTitle className="text-xl">{plan.title}</CardTitle>
              <p className="text-sm text-muted-foreground">{plan.goal}</p>
            </div>
            <Badge variant={STATUS_VARIANT[plan.status as StepStatus] ?? "outline"}>
              {plan.status.replace("_", " ")}
            </Badge>
          </div>
        </CardHeader>
        <CardContent className="flex flex-wrap items-center gap-x-4 gap-y-1.5 font-mono text-xs text-muted-foreground">
          <span>
            id <span className="text-foreground/80">{plan.id.slice(0, 8)}</span>
          </span>
          <Separator orientation="vertical" className="h-3" />
          <span>
            session <span className="text-foreground/80">{plan.session_id.slice(0, 8)}</span>
          </span>
          <Separator orientation="vertical" className="h-3" />
          <span>
            {plan.steps.length} step{plan.steps.length === 1 ? "" : "s"}
          </span>
          {totalCriteria > 0 && (
            <>
              <Separator orientation="vertical" className="h-3" />
              <span>
                {verifiedCount}/{totalCriteria} criteria verified
              </span>
            </>
          )}
        </CardContent>
      </Card>

      {/* Task */}
      <Section title="Task">
        <p className="text-sm leading-relaxed text-foreground/90">{plan.task}</p>
      </Section>

      {/* Success criteria */}
      {plan.success_criteria.length > 0 && (
        <Section
          title="Success criteria"
          count={`${verifiedCount}/${plan.success_criteria.length}`}
        >
          <ul className="space-y-1.5">
            {plan.success_criteria.map((c) => (
              <li
                key={c.id}
                className={`flex items-start gap-2.5 rounded-md border border-border/60 bg-card/40 px-3 py-2 text-sm ${
                  c.verified ? "text-emerald-400 line-through" : "text-foreground/90"
                }`}
              >
                <span className="mt-0.5 font-mono text-xs text-muted-foreground">
                  {c.verified ? "☑" : "☐"}
                </span>
                <span>{c.text}</span>
              </li>
            ))}
          </ul>
        </Section>
      )}

      {/* Assumptions */}
      {plan.assumptions.length > 0 && (
        <Section title="Assumptions">
          <ul className="space-y-1.5">
            {plan.assumptions.map((a) => (
              <li
                key={a.id}
                className={`flex items-start gap-2.5 rounded-md border bg-card/40 px-3 py-2 text-sm ${
                  a.critical ? "border-l-2 border-l-amber-500 border-border/60" : "border-border/60"
                }`}
              >
                {a.critical && (
                  <Badge variant="warning" className="shrink-0">critical</Badge>
                )}
                <span className="text-foreground/90">{a.text}</span>
              </li>
            ))}
          </ul>
        </Section>
      )}

      {/* Unknowns */}
      {plan.unknowns.length > 0 && (
        <Section title="Unknowns">
          <ul className="space-y-1.5">
            {plan.unknowns.map((u) => (
              <li
                key={u.id}
                className={`flex items-start gap-2.5 rounded-md border bg-card/40 px-3 py-2 text-sm ${
                  u.blocking ? "border-l-2 border-l-red-500 border-border/60" : "border-border/60"
                }`}
              >
                {u.blocking && (
                  <Badge variant="danger" className="shrink-0">blocking</Badge>
                )}
                <span className="text-foreground/90">{u.text}</span>
              </li>
            ))}
          </ul>
        </Section>
      )}

      {/* Steps */}
      <Section title="Steps" count={String(plan.steps.length)}>
        <ol className="space-y-3">
          {plan.steps.map((step) => (
            <StepCard
              key={step.id}
              step={step}
              onDelta={(action) => {
                if (!onDelta) return;
                onDelta({
                  id: crypto.randomUUID(),
                  timestamp: new Date().toISOString(),
                  target: { step: { plan_id: plan.id, step_id: step.id } },
                  action,
                  comment: null,
                });
              }}
            />
          ))}
        </ol>
      </Section>

      {/* Plan actions */}
      {onDelta && (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Plan actions</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-wrap gap-2">
            <Button
              onClick={() =>
                onDelta({
                  id: crypto.randomUUID(),
                  timestamp: new Date().toISOString(),
                  target: { plan: plan.id },
                  action: { type: "approve_plan" } satisfies DeltaAction,
                  comment: null,
                })
              }
            >
              <Check className="h-4 w-4" /> Approve plan
            </Button>
            <Button
              variant="outline"
              onClick={() => {
                const reason = window.prompt("Reason for rejection?");
                if (!reason) return;
                onDelta({
                  id: crypto.randomUUID(),
                  timestamp: new Date().toISOString(),
                  target: { plan: plan.id },
                  action: { type: "reject_plan", reason } satisfies DeltaAction,
                  comment: null,
                });
              }}
            >
              <X className="h-4 w-4" /> Reject
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                const reason = window.prompt("Reason to abort?");
                if (!reason) return;
                onDelta({
                  id: crypto.randomUUID(),
                  timestamp: new Date().toISOString(),
                  target: { plan: plan.id },
                  action: { type: "abort_plan", reason } satisfies DeltaAction,
                  comment: null,
                });
              }}
            >
              <CircleDashed className="h-4 w-4" /> Abort
            </Button>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function Section({
  title,
  count,
  children,
}: {
  title: string;
  count?: string;
  children: React.ReactNode;
}) {
  return (
    <section className="space-y-3">
      <div className="flex items-center gap-2">
        <h2 className="font-mono text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
          {title}
        </h2>
        {count && (
          <Badge variant="secondary" className="font-mono">
            {count}
          </Badge>
        )}
      </div>
      {children}
    </section>
  );
}

interface StepProps {
  step: Step;
  onDelta: (action: DeltaAction) => void;
}

function StepCard({ step, onDelta }: StepProps) {
  const Icon = INTENT_ICON[step.intent];

  return (
    <li
      className={`group relative overflow-hidden rounded-lg border border-border/60 bg-card/40 pl-4 pr-4 py-3 transition-colors hover:bg-card/70`}
    >
      {/* Status stripe */}
      <div
        className={`absolute left-0 top-0 h-full w-1 ${STATUS_DOT[step.status]}`}
        aria-hidden
      />

      <div className="flex items-start gap-3">
        <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full border border-border bg-background font-mono text-[11px] text-muted-foreground">
          {step.order + 1}
        </div>
        <div className="min-w-0 flex-1 space-y-2">
          {/* Header */}
          <div className="flex flex-wrap items-center gap-2">
            <Icon className="h-3.5 w-3.5 text-muted-foreground" />
            <span className="font-medium text-foreground">{step.title}</span>
            <Badge variant="outline" className="font-mono text-[10px]">
              {INTENT_LABEL[step.intent]}
            </Badge>
            <Badge
              variant={STATUS_VARIANT[step.status]}
              className="ml-auto font-mono text-[10px]"
            >
              {step.status.replace("_", " ")}
            </Badge>
          </div>

          {/* Body */}
          <p className="text-sm leading-relaxed text-foreground/85">{step.what}</p>

          {(step.how || step.risk || step.backout) && (
            <div className="grid gap-1.5 pt-1">
              {step.how && <StepDetail label="How" body={step.how} tone="muted" />}
              {step.risk && <StepDetail label="Risk" body={step.risk} tone="warning" />}
              {step.backout && (
                <StepDetail label="Backout" body={step.backout} tone="info" />
              )}
            </div>
          )}

          {/* Retrospective */}
          {step.retrospective && (
            <div className="rounded-md border border-emerald-500/30 bg-emerald-500/5 p-3">
              <div className="mb-1 flex items-center gap-2">
                <span className="font-mono text-[10px] font-semibold uppercase tracking-wider text-emerald-400">
                  Retrospective
                </span>
                <Badge
                  variant={
                    step.retrospective.matches_plan === "match"
                      ? "success"
                      : step.retrospective.matches_plan === "partial"
                        ? "warning"
                        : "destructive"
                  }
                  className="text-[10px]"
                >
                  {step.retrospective.matches_plan}
                </Badge>
              </div>
              <p className="text-sm text-foreground/85">{step.retrospective.done}</p>
              {step.retrospective.deviation && (
                <p className="mt-1.5 text-xs text-amber-400">
                  <strong className="font-semibold">Deviation:</strong>{" "}
                  {step.retrospective.deviation}
                </p>
              )}
            </div>
          )}

          {/* Actions */}
          {step.status === "pending" && (
            <div className="flex flex-wrap items-center gap-1.5 pt-1">
              <Button
                size="sm"
                onClick={() => onDelta({ type: "approve_step" } satisfies DeltaAction)}
              >
                <Check className="h-3.5 w-3.5" /> Approve
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  const reason = window.prompt("Why reject this step?");
                  if (!reason) return;
                  onDelta({ type: "reject_step", reason } satisfies DeltaAction);
                }}
              >
                <X className="h-3.5 w-3.5" /> Reject
              </Button>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => {
                  const reason = window.prompt("Why skip?");
                  if (!reason) return;
                  onDelta({ type: "skip_step", reason } satisfies DeltaAction);
                }}
              >
                <SkipForward className="h-3.5 w-3.5" /> Skip
              </Button>
              <StepEditDialog step={step} onApply={(action) => onDelta(action)} />
            </div>
          )}

          {step.status === "in_progress" && (
            <div className="flex flex-wrap items-center gap-1.5 pt-1">
              <Button
                size="sm"
                onClick={() => {
                  const actual = window.prompt("What was actually done?");
                  if (!actual) return;
                  onDelta({
                    type: "complete_step",
                    actual,
                    matches_plan: "match",
                    deviation: null,
                  } satisfies DeltaAction);
                }}
              >
                <Check className="h-3.5 w-3.5" /> Complete
              </Button>
              <Button
                size="sm"
                variant="destructive"
                onClick={() => {
                  const error = window.prompt("What went wrong?");
                  if (!error) return;
                  onDelta({ type: "fail_step", error } satisfies DeltaAction);
                }}
              >
                <X className="h-3.5 w-3.5" /> Fail
              </Button>
            </div>
          )}
        </div>
      </div>
    </li>
  );
}

function StepDetail({
  label,
  body,
  tone,
}: {
  label: string;
  body: string;
  tone: "muted" | "warning" | "info";
}) {
  const toneClass =
    tone === "warning"
      ? "border-amber-500/20 bg-amber-500/5"
      : tone === "info"
        ? "border-sky-500/20 bg-sky-500/5"
        : "border-border/60 bg-background/30";

  return (
    <details className={`group/details rounded-md border ${toneClass} px-3 py-2 text-xs`}>
      <summary className="cursor-pointer select-none text-muted-foreground transition-colors hover:text-foreground [&::-webkit-details-marker]:hidden">
        <span className="inline-flex items-center gap-1.5 font-mono text-[10px] font-semibold uppercase tracking-wider">
          {label}
          <ChevronDown className="h-3 w-3 transition-transform group-open/details:rotate-180" />
        </span>
      </summary>
      <p className="mt-1.5 leading-relaxed text-foreground/85">{body}</p>
    </details>
  );
}

function StepEditDialog({
  step,
  onApply,
}: {
  step: Step;
  onApply: (action: DeltaAction) => void;
}) {
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState(step.title);
  const [what, setWhat] = useState(step.what);
  const [risk, setRisk] = useState(step.risk ?? "");
  const [backout, setBackout] = useState(step.backout ?? "");

  const reset = () => {
    setTitle(step.title);
    setWhat(step.what);
    setRisk(step.risk ?? "");
    setBackout(step.backout ?? "");
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (next) reset();
      }}
    >
      <Button size="sm" variant="ghost" onClick={() => setOpen(true)}>
        <Pencil className="h-3.5 w-3.5" /> Edit
      </Button>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>Edit step</DialogTitle>
          <DialogDescription>
            Each field becomes a structured <code>Delta</code>. The LLM will
            re-render the plan on the next turn.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-3">
          <Field label="title">
            <Input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
            />
          </Field>
          <Field label="what">
            <Textarea
              value={what}
              onChange={(e) => setWhat(e.target.value)}
              rows={3}
            />
          </Field>
          <Field label="risk">
            <Input
              value={risk}
              onChange={(e) => setRisk(e.target.value)}
              placeholder="What could go wrong?"
            />
          </Field>
          <Field label="backout">
            <Input
              value={backout}
              onChange={(e) => setBackout(e.target.value)}
              placeholder="How to roll back if this fails"
            />
          </Field>
        </div>
        <DialogFooter>
          <DialogClose asChild>
            <Button variant="outline">Cancel</Button>
          </DialogClose>
          <Button
            onClick={() => {
              if (title !== step.title) {
                onApply({ type: "set_step_title", text: title } satisfies DeltaAction);
              }
              if (what !== step.what) {
                onApply({ type: "set_step_what", text: what } satisfies DeltaAction);
              }
              if (risk !== (step.risk ?? "")) {
                onApply({ type: "set_step_risk", text: risk || null } satisfies DeltaAction);
              }
              if (backout !== (step.backout ?? "")) {
                onApply({ type: "set_step_backout", text: backout || null } satisfies DeltaAction);
              }
              setOpen(false);
            }}
          >
            Apply as Delta
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <label className="grid gap-1.5">
      <span className="font-mono text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        {label}
      </span>
      {children}
    </label>
  );
}
