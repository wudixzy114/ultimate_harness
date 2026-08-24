// Code viewer types — intentionally hand-mirrored from the Rust analyzer's
// JSON schema (see tools/uh-devtool-analyze/src/model.rs). Do NOT import
// from src/types/* — this is a dev tool, isolated from the product
// type mirrors.

export interface DataFlow {
  input?: string;
  output?: string;
  side_effects?: string[];
  depends_on?: string[];
}

export type PubKind =
  | "struct"
  | "enum"
  | "trait"
  | "fn"
  | "const"
  | "type"
  | "impl"
  | "mod";

export interface FieldInfo {
  name: string;
  ty: string;
  line: number;
  has_doc: boolean;
  doc?: string;
  vis: string;
}

export interface PubItem {
  kind: PubKind;
  name: string;
  line: number;
  tags?: string[];
  invariants?: string[];
  data_flow?: DataFlow;
  children?: PubItem[];
  fields?: FieldInfo[];
  has_undocumented_fields?: boolean;
}

export interface TestInfo {
  name: string;
  line: number;
  attrs: { tags?: string[]; invariants?: string[]; data_flow?: DataFlow };
}

export interface FnAnalysis {
  name: string;
  line: number;
  attrs: { tags?: string[]; invariants?: string[]; data_flow?: DataFlow };
}

export interface FileAnalysis {
  path: string;
  purpose: string;
  file_tags?: string[];
  file_invariants?: string[];
  tests: TestInfo[];
  exposed: PubItem[];
  fns: FnAnalysis[];
}

export interface FileSummary {
  path: string;
  purpose: string;
  file_tags: string[];
  exposed_count: number;
  test_count: number;
}
