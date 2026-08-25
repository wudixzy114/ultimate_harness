import { useEffect, useMemo, useState } from "react";
import { fetchFile, fetchIndex } from "./api";
import type { FileAnalysis, PubItem } from "./types";
import { FileTree } from "./FileTree";
import { ExposedTree } from "./ExposedTree";
import { DetailPanel } from "./DetailPanel";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { AlertCircle, ArrowLeft, Code2, Loader2, AlertTriangle } from "lucide-react";

export function Review() {
  // FileSummary list (drives the left tree) — separate from the
  // currently-selected file's full analysis.
  const [fileSummaries, setFileSummaries] = useState<import("./types").FileSummary[] | null>(null);
  const [filesErr, setFilesErr] = useState<string | null>(null);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [current, setCurrent] = useState<FileAnalysis | null>(null);
  const [currentErr, setCurrentErr] = useState<string | null>(null);
  const [selectedItem, setSelectedItem] = useState<string | null>(null);

  // initial index load
  useEffect(() => {
    fetchIndex()
      .then((idx) => {
        setFileSummaries(idx);
        if (idx.length > 0) setSelectedFile(idx[0].path);
      })
      .catch((e) => setFilesErr(String(e)));
  }, []);

  // poll index every 2s
  useEffect(() => {
    const t = setInterval(() => {
      fetchIndex()
        .then((idx) => setFileSummaries(idx))
        .catch(() => undefined);
    }, 2000);
    return () => clearInterval(t);
  }, []);

  // fetch full file when selectedFile changes
  useEffect(() => {
    if (!selectedFile) {
      setCurrent(null);
      return;
    }
    setCurrentErr(null);
    fetchFile(selectedFile)
      .then((a) => {
        setCurrent(a);
        setSelectedItem(null);
      })
      .catch((e) => setCurrentErr(String(e)));
  }, [selectedFile]);

  // poll current file
  useEffect(() => {
    if (!selectedFile) return;
    const t = setInterval(() => {
      fetchFile(selectedFile)
        .then((a) => setCurrent(a))
        .catch(() => undefined);
    }, 2000);
    return () => clearInterval(t);
  }, [selectedFile]);

  const selectedItemObject = useMemo<PubItem | null>(() => {
    if (!current || !selectedItem) return null;
    return findByName(current.exposed, selectedItem);
  }, [current, selectedItem]);

  const detailSubject = useMemo(() => {
    if (!current) return null;
    if (selectedItemObject) {
      return {
        kind: "item" as const,
        item: selectedItemObject,
        onSelectNested: setSelectedItem,
      };
    }
    return {
      kind: "file" as const,
      purpose: current.purpose,
      file_tags: current.file_tags,
      file_invariants: current.file_invariants,
      tests: current.tests,
      fns: current.fns,
    };
  }, [current, selectedItemObject]);

  return (
    <div className="flex h-full flex-col">
      <Header
        fileCount={fileSummaries?.length}
        error={filesErr}
        onBack={() => {
          window.location.hash = "";
        }}
      />

      <main className="grid min-h-0 flex-1 grid-cols-[280px_1fr_1fr] divide-x divide-border/60">
        {/* LEFT: file tree */}
        <aside className="flex flex-col overflow-hidden border-r border-border/60 bg-card/20">
          {fileSummaries ? (
            <FileTree
              files={fileSummaries}
              selected={selectedFile}
              onSelect={setSelectedFile}
            />
          ) : (
            <div className="flex flex-1 items-center justify-center gap-2 font-mono text-xs text-muted-foreground">
              <Loader2 className="h-3.5 w-3.5 animate-spin" /> loading index…
            </div>
          )}
        </aside>

        {/* CENTER: exposed tree */}
        <section className="flex flex-col overflow-hidden bg-background/30">
          {currentErr ? (
            <div className="flex flex-1 items-center justify-center gap-2 p-4 font-mono text-xs text-red-400">
              <AlertCircle className="h-4 w-4" /> error: {currentErr}
            </div>
          ) : !current ? (
            <div className="flex flex-1 items-center justify-center font-mono text-xs text-muted-foreground">
              select a file
            </div>
          ) : (
            <>
              <CenterHeader current={current} />
              <ScrollArea className="flex-1">
                <ExposedTree
                  items={current.exposed}
                  selected={selectedItem}
                  onSelect={setSelectedItem}
                />
              </ScrollArea>
            </>
          )}
        </section>

        {/* RIGHT: detail panel */}
        <section className="flex flex-col overflow-hidden bg-card/10">
          <DetailPanel subject={detailSubject} />
        </section>
      </main>
    </div>
  );
}

function Header({
  fileCount,
  error,
  onBack,
}: {
  fileCount: number | undefined;
  error: string | null;
  onBack: () => void;
}) {
  return (
    <header className="flex items-center justify-between border-b border-border bg-card/40 px-5 py-2.5">
      <div className="flex items-center gap-3">
        <div className="flex h-7 w-7 items-center justify-center rounded-md bg-cyan-500/10 text-cyan-400 ring-1 ring-cyan-500/20">
          <Code2 className="h-3.5 w-3.5" />
        </div>
        <div>
          <h1 className="text-sm font-semibold leading-none tracking-tight">
            code viewer
          </h1>
          <p className="mt-0.5 font-mono text-[10px] text-muted-foreground">
            /review · 2s polling
          </p>
        </div>
        {fileCount !== undefined && (
          <Badge variant="secondary" className="font-mono text-[10px]">
            {fileCount} files
          </Badge>
        )}
        {error && (
          <Badge variant="destructive" className="font-mono text-[10px]">
            <AlertCircle className="h-2.5 w-2.5" /> {error}
          </Badge>
        )}
      </div>
      <Button
        size="sm"
        variant="outline"
        onClick={onBack}
        className="h-7 gap-1.5 px-2 text-[11px]"
      >
        <ArrowLeft className="h-3 w-3" /> back to product
      </Button>
    </header>
  );
}

function CenterHeader({ current }: { current: FileAnalysis }) {
  const hasUndoc = current.exposed.some((i) => i.has_undocumented_fields);
  return (
    <div className="flex items-center justify-between gap-2 border-b border-border/60 bg-background/40 px-3 py-2">
      <code className="truncate font-mono text-[11px] text-foreground/85">
        {current.path}
      </code>
      {hasUndoc && (
        <span className="inline-flex shrink-0 items-center gap-1 rounded bg-red-500/15 px-1.5 py-0.5 font-mono text-[10px] text-red-400">
          <AlertTriangle className="h-2.5 w-2.5" />
          undoc fields
        </span>
      )}
    </div>
  );
}

function findByName(items: PubItem[], name: string): PubItem | null {
  for (const it of items) {
    if (it.name === name) return it;
    if (it.children) {
      const found = findByName(it.children, name);
      if (found) return found;
    }
  }
  return null;
}
