// TypeScript mirror of uh-core::schema::context

import type { SessionId } from "./plan";

export type LayerKind =
  | "world_memory"
  | "project_memory"
  | "session_memory"
  | "working_memory"
  | "step_memory";

export interface ModelInfo {
  provider: string;
  model: string;
  supports_caching: boolean;
  cache_breakpoints: number;
}

export interface CacheBoundary {
  prefix_tokens: number;
  prefix_hash: string;
  provider_cache_supported: boolean;
  cache_hits_estimated: number;
}

export interface ContextLayer {
  layer: LayerKind;
  tokens: number;
  token_budget: number;
  item_count: number;
  is_cacheable: boolean;
}

export interface ContextSnapshot {
  session_id: SessionId;
  timestamp: string;
  model: ModelInfo;
  cache_boundary: CacheBoundary;
  layers: ContextLayer[];
  total_input_tokens: number;
  max_context_window: number;
}
