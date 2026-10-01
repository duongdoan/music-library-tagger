// Typed wrappers around the Rust commands (src-tauri/src/commands.rs).
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Fields = Record<string, string>;

export interface TrackRow {
  path: string;
  sourceId: number;
  dir: string;
  file: string;
  ext: string;
  size: number;
  mtime: number;
  fields: Fields;
  hasArt: boolean;
  format: string;
  durationMs: number | null;
  riffMismatch: boolean;
  error: string | null;
  readonly: boolean;
}

export interface Source {
  id: number;
  path: string;
  name: string;
  excludes: string[];
  status: string;
  lastScan: number | null;
}

export interface StagedChange { path: string; field: string; orig: string; value: string }
export interface StageInput { path: string; field: string; value: string }
export interface ApplyItem { path: string; field: string }

export interface ScanProgress { sourceId: number; phase: "list" | "read"; done: number; total: number; perSec: number }
export interface ScanSummary {
  sourceId: number; read: number; added: number; changed: number; removed: number;
  errors: number; skipped: number; cancelled: boolean; unavailable: boolean;
}
export interface FileResult { path: string; status: "ok" | "conflict" | "error" | "cancelled"; message: string | null; row: TrackRow | null }
export interface ApplyReport { runId: number; ok: number; conflict: number; error: number; cancelled: number; results: FileResult[] }
export interface RunInfo {
  id: number; kind: string; label: string | null; started: number; finished: number | null; status: string;
  filesOk: number; filesConflict: number; filesError: number; summary: string; undoOf: number | null;
}
export interface UndoPlan {
  runId: number;
  stage: StageInput[];
  changedAfter: { path: string; field: string; current: string; before: string }[];
  missing: string[];
}

export const api = {
  listSources: () => invoke<Source[]>("list_sources"),
  addSource: (path: string) => invoke<Source>("add_source", { path }),
  removeSource: (id: number) => invoke<void>("remove_source", { id }),
  setExcludes: (id: number, excludes: string[]) => invoke<void>("set_excludes", { id, excludes }),
  scanSource: (id: number, folder?: string) => invoke<ScanSummary>("scan_source", { id, folder: folder ?? null }),
  cancelScan: () => invoke<void>("cancel_scan"),
  loadTracks: () => invoke<TrackRow[]>("load_tracks"),
  stageSet: (changes: StageInput[]) => invoke<{ applied: number; rejected: string[] }>("stage_set", { changes }),
  stageList: () => invoke<StagedChange[]>("stage_list"),
  stageDiscard: (keys: ApplyItem[]) => invoke<void>("stage_discard", { keys }),
  applyRun: (items: ApplyItem[], undoOf?: number) => invoke<ApplyReport>("apply_run", { items, undoOf: undoOf ?? null }),
  cancelApply: () => invoke<void>("cancel_apply"),
  listRuns: () => invoke<RunInfo[]>("list_runs"),
  undoPlan: (runId: number) => invoke<UndoPlan>("undo_plan", { runId }),
  getSetting: (key: string) => invoke<string | null>("get_setting", { key }),
  setSetting: (key: string, value: string) => invoke<void>("set_setting", { key, value }),
};

export const on = {
  scanProgress: (f: (p: ScanProgress) => void): Promise<UnlistenFn> => listen<ScanProgress>("scan-progress", (e) => f(e.payload)),
  scanBatch: (f: (b: { rows: TrackRow[]; removed: string[] }) => void): Promise<UnlistenFn> =>
    listen<{ rows: TrackRow[]; removed: string[] }>("scan-batch", (e) => f(e.payload)),
  applyProgress: (f: (p: { done: number; total: number; result: FileResult }) => void): Promise<UnlistenFn> =>
    listen<{ done: number; total: number; result: FileResult }>("apply-progress", (e) => f(e.payload)),
};
