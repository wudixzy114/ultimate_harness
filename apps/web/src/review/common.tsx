// Shared layout primitives for all right-side panels.
// Migrated to shadcn/Tailwind: no external CSS file.

import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";

export function Section({
  title,
  children,
  count,
}: {
  title: string;
  children: React.ReactNode;
  count?: React.ReactNode;
}) {
  return (
    <section className="space-y-2">
      <div className="flex items-center gap-2">
        <h3 className="font-mono text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
          {title}
        </h3>
        {count}
      </div>
      <div>{children}</div>
    </section>
  );
}

export function TagList({ tags }: { tags: string[] }) {
  return (
    <div className="flex flex-wrap items-center gap-1.5">
      {tags.map((t) => (
        <Badge key={t} variant="secondary" className="font-mono text-[10px]">
          {t}
        </Badge>
      ))}
    </div>
  );
}

export function Divider() {
  return <Separator className="my-3" />;
}
