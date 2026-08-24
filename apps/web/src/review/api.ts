// HTTP client for the uh-devtool analyzer.
// Vite dev server proxies `/api/*` to `http://127.0.0.1:7700` (see vite.config.ts).

import type { FileAnalysis, FileSummary } from "./types";

const BASE = "/api";

export async function fetchIndex(): Promise<FileSummary[]> {
  const r = await fetch(`${BASE}/index`);
  if (!r.ok) throw new Error(`index: ${r.status}`);
  return r.json();
}

export async function fetchFile(path: string): Promise<FileAnalysis> {
  const r = await fetch(`${BASE}/file?path=${encodeURIComponent(path)}`);
  if (!r.ok) throw new Error(`file ${path}: ${r.status}`);
  return r.json();
}

export function fileGroup(path: string): string {
  // First segment (e.g. "uh-core", "uh-skill-plan-first", "uh").
  const seg = path.replace(/\\/g, "/").split("/")[0] ?? "(other)";
  return seg;
}
