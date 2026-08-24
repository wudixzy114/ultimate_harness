import { useEffect, useMemo, useState } from "react";
import "./styles.css";
import { fetchFile, fetchIndex } from "./api";
import type { FileAnalysis, FileSummary, PubItem } from "./types";
import { FileTree } from "./FileTree";
import { ExposedTree } from "./ExposedTree";
import { FieldTable } from "./FieldTable";
import { DetailPanel } from "./DataFlowPanel";

export function Review() {
  const [files, setFiles] = useState<FileSummary[] | null>(null);
  const [filesErr, setFilesErr] = useState<string | null>(null);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [current, setCurrent] = useState<FileAnalysis | null>(null);
  const [currentErr, setCurrentErr] = useState<string | null>(null);
  const [selectedItem, setSelectedItem] = useState<string | null>(null);
  const [selectedField, setSelectedField] = useState<string | null>(null);

  // initial index load
  useEffect(() => {
    fetchIndex()
      .then((idx) => {
        setFiles(idx);
        if (idx.length > 0) setSelectedFile(idx[0].path);
      })
      .catch((e) => setFilesErr(String(e)));
  }, []);

  // poll index every 2s (cheap; MVP — no SSE yet)
  useEffect(() => {
    const t = setInterval(() => {
      fetchIndex()
        .then((idx) => setFiles(idx))
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
        setSelectedField(null);
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
    if (selectedItemObject) return { kind: "item" as const, item: selectedItemObject };
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
    <div className="review">
      <header className="review__header">
        <h1 className="review__title">
          code viewer <span className="review__title-sub">/review</span>
        </h1>
        <div className="review__meta">
          {files ? `${files.length} files` : "loading…"}
          {filesErr ? <span className="review__err"> · {filesErr}</span> : null}
          <a className="review__back" href={window.location.pathname}>
            ← back to product
          </a>
        </div>
      </header>
      <main className="review__main">
        <aside className="review__left">
          {files ? (
            <FileTree
              files={files}
              selected={selectedFile}
              onSelect={setSelectedFile}
            />
          ) : (
            <div className="review-loading">loading index…</div>
          )}
        </aside>
        <section className="review__center">
          {currentErr ? (
            <div className="review-err">error: {currentErr}</div>
          ) : !current ? (
            <div className="review-loading">select a file</div>
          ) : (
            <>
              <div className="review__center-head">
                <code className="review__path">{current.path}</code>
                {current.exposed.some((i) => i.has_undocumented_fields) ? (
                  <span className="review__badge review__badge--undoc">
                    undoc fields present
                  </span>
                ) : null}
              </div>
              <ExposedTree
                items={current.exposed}
                selected={selectedItem}
                onSelect={setSelectedItem}
              />
            </>
          )}
        </section>
        <section className="review__right">
          {selectedItemObject ? (
            <>
              <div className="review__right-head">
                {selectedItemObject.kind} {selectedItemObject.name}
                {selectedItemObject.fields && selectedItemObject.fields.length > 0
                  ? ` · ${selectedItemObject.fields.length} fields`
                  : ""}
              </div>
              <FieldTable
                item={selectedItemObject}
                selectedField={selectedField}
                onSelectField={setSelectedField}
              />
            </>
          ) : null}
          <DetailPanel subject={detailSubject} />
        </section>
      </main>
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
