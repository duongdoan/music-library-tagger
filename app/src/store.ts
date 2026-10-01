// App state: one small external store, read with useStore(). Staged edits are
// mirrored here and persisted in SQLite through api.stageSet (APP-STAGE-R5).
import { useSyncExternalStore } from "react";
import { api, type Source, type StagedChange, type TrackRow, type StageInput, type ScanProgress } from "./api";
import { AA_SYNC, fold, isPseudo, normalize, PLACEHOLDERS, RIFF_SYNC, TEXT_FIELDS, validate } from "./fields";

export type Staged = { orig: string; value: string };
type Change = { key: string; path: string; field: string; prev?: string; next?: string };
type Op = { label: string; ch: Change[] };

export interface QuickFilter { key: string; label: string; test: (t: TrackRow, ctx: FilterCtx) => boolean }
interface FilterCtx { inconsistent: Set<string> }

export const K = (path: string, field: string) => path + "\u0001" + field;

class Store {
  version = 0;
  sources: Source[] = [];
  tracks: TrackRow[] = [];
  byPath = new Map<string, TrackRow>();
  staged = new Map<string, Staged>();
  undoStack: Op[] = [];
  redoStack: Op[] = [];
  selected: string[] = [];
  folder: string | null = null;
  quick = "all";
  query = "";
  sort: { field: string; desc: boolean } | null = null;
  scan: (ScanProgress & { name: string }) | null = null;
  status = new Map<string, { status: string; message: string | null }>();
  toast: { text: string; id: number } | null = null;
  lastFill: { op: Op; redo: (mode: "copy" | "series") => void; mode: "copy" | "series" } | null = null;
  private listeners = new Set<() => void>();
  private visibleCache: { v: number; rows: TrackRow[] } | null = null;

  subscribe = (f: () => void) => {
    this.listeners.add(f);
    return () => this.listeners.delete(f);
  };
  emit() {
    this.version++;
    this.listeners.forEach((f) => f());
  }

  // ---------------------------------------------------------------- data
  async init() {
    const [sources, tracks, staged] = await Promise.all([api.listSources(), api.loadTracks(), api.stageList()]);
    this.sources = sources;
    this.setTracks(tracks);
    this.setStaged(staged);
    this.emit();
  }
  setTracks(rows: TrackRow[]) {
    this.tracks = rows;
    this.byPath = new Map(rows.map((r) => [r.path, r]));
  }
  setStaged(list: StagedChange[]) {
    this.staged = new Map(list.map((s) => [K(s.path, s.field), { orig: s.orig, value: s.value }]));
  }
  mergeRows(rows: TrackRow[], removed: string[]) {
    for (const r of rows) this.byPath.set(r.path, r);
    for (const p of removed) this.byPath.delete(p);
    this.tracks = [...this.byPath.values()].sort((a, b) => (a.dir === b.dir ? a.file.localeCompare(b.file) : a.dir.localeCompare(b.dir)));
    this.emit();
  }
  async refreshSources() {
    this.sources = await api.listSources();
    this.emit();
  }

  // ---------------------------------------------------------------- values
  val(t: TrackRow, field: string): string {
    return this.staged.get(K(t.path, field))?.value ?? t.fields[field] ?? "";
  }
  isPending(t: TrackRow, field: string) {
    return this.staged.has(K(t.path, field));
  }
  hasPending(t: TrackRow) {
    for (const k of this.staged.keys()) if (k.startsWith(t.path + "\u0001")) return true;
    return false;
  }
  editable(t: TrackRow) {
    const src = this.sources.find((s) => s.id === t.sourceId);
    return !t.error && !t.readonly && src?.status !== "unavailable";
  }
  pendingFiles() {
    return new Set([...this.staged.keys()].map((k) => k.split("\u0001")[0])).size;
  }

  // ---------------------------------------------------------------- editing (APP-STAGE)
  /** Stage values; returns {applied, skipped, invalid}. One call = one undo step (APP-STAGE-R6). */
  edit(inputs: StageInput[], label: string): { applied: number; skipped: number; invalid: number } {
    const ch: Change[] = [];
    let skipped = 0, invalid = 0;
    const seen = new Set<string>();
    for (const i of inputs) {
      const t = this.byPath.get(i.path);
      if (!t || !this.editable(t)) { skipped++; continue; }
      const v = isPseudo(i.field) ? i.value : normalize(i.value);
      if (validate(i.field, v)) { invalid++; continue; }
      const key = K(i.path, i.field);
      if (seen.has(key)) continue;
      seen.add(key);
      const prev = this.staged.get(key)?.value;
      const orig = t.fields[i.field] ?? "";
      const next = v === orig ? undefined : v;
      if (next === prev) continue;
      ch.push({ key, path: i.path, field: i.field, prev, next });
    }
    if (ch.length) {
      const op = { label, ch };
      this.apply(op, 1);
      this.undoStack.push(op);
      this.redoStack = [];
      this.lastFill = null;
    }
    this.emit();
    return { applied: ch.length, skipped, invalid };
  }
  private apply(op: Op, dir: 1 | -1) {
    const send: StageInput[] = [];
    for (const c of op.ch) {
      const v = dir === 1 ? c.next : c.prev;
      const t = this.byPath.get(c.path)!;
      const orig = t.fields[c.field] ?? "";
      if (v === undefined) this.staged.delete(c.key);
      else this.staged.set(c.key, { orig, value: v });
      send.push({ path: c.path, field: c.field, value: v ?? orig });
    }
    api.stageSet(send).catch((e) => this.notify(String(e)));
  }
  undo() {
    const op = this.undoStack.pop();
    if (!op) return;
    this.apply(op, -1);
    this.redoStack.push(op);
    this.lastFill = null;
    this.notify("Đã hoàn tác: " + op.label);
  }
  redo() {
    const op = this.redoStack.pop();
    if (!op) return;
    this.apply(op, 1);
    this.undoStack.push(op);
    this.notify("Làm lại: " + op.label);
  }
  /** Replace the last fill with another mode without adding an undo step (APP-STAGE-R16). */
  undoSilently(op: Op) {
    if (this.undoStack[this.undoStack.length - 1] !== op) return false;
    this.undoStack.pop();
    this.apply(op, -1);
    return true;
  }
  lastOp() {
    return this.undoStack[this.undoStack.length - 1];
  }
  discard(keys: string[], label: string) {
    const inputs = keys.map((k) => {
      const [path, field] = k.split("\u0001");
      return { path, field, value: this.byPath.get(path)?.fields[field] ?? "" };
    });
    // pseudo rows (RIFF / Album Artist key sync) have no field value: remove them directly
    const pseudo = keys.filter((k) => isPseudo(k.split("\u0001")[1]));
    if (pseudo.length) {
      pseudo.forEach((k) => this.staged.delete(k));
      api.stageDiscard(pseudo.map((k) => { const [path, field] = k.split("\u0001"); return { path, field }; }));
    }
    this.edit(inputs.filter((i) => !isPseudo(i.field)), label);
  }

  // ---------------------------------------------------------------- view
  setSelected(paths: string[]) {
    this.selected = paths;
    this.emit();
  }
  setView(p: Partial<Pick<Store, "folder" | "quick" | "query" | "sort">>) {
    Object.assign(this, p);
    this.emit();
  }
  notify(text: string) {
    this.toast = { text, id: Date.now() };
    this.emit();
  }

  quickFilters: QuickFilter[] = [
    { key: "all", label: "Tất cả", test: () => true },
    { key: "pending", label: "Có thay đổi chờ", test: (t) => this.hasPending(t) },
    { key: "err", label: "Lỗi", test: (t) => !!t.error || this.status.has(t.path) },
    { key: "noaa", label: "Thiếu Album Artist", test: (t) => this.editable(t) && !this.val(t, "albumartist") },
    { key: "noalbum", label: "Thiếu Album", test: (t) => this.editable(t) && !this.val(t, "album") },
    { key: "notrack", label: "Thiếu Track", test: (t) => this.editable(t) && !this.val(t, "track") },
    { key: "noart", label: "Thiếu ảnh bìa", test: (t) => this.editable(t) && !t.hasArt },
    { key: "incons", label: "Album không đồng nhất", test: (t, c) => this.editable(t) && c.inconsistent.has(t.dir) },
    { key: "placeholder", label: "Giá trị giữ chỗ", test: (t) => this.editable(t) && ["artist", "album", "title", "albumartist"].some((f) => PLACEHOLDERS.test(this.val(t, f))) },
    { key: "space", label: "Có khoảng trắng thừa", test: (t) => this.editable(t) && TEXT_FIELDS.some((f) => { const v = this.val(t, f); return v !== v.replace(/\s+/g, " ").trim(); }) },
    { key: "nfd", label: "Không chuẩn Unicode", test: (t) => this.editable(t) && TEXT_FIELDS.some((f) => { const v = this.val(t, f); return v !== v.normalize("NFC"); }) },
    { key: "aa", label: "Album Artist lệch giữa các khoá", test: (t) => t.aaMismatch && !this.staged.has(K(t.path, AA_SYNC)) && !this.isPending(t, "albumartist") },
    { key: "riff", label: "WAV: RIFF INFO lệch", test: (t) => t.riffMismatch && !this.staged.has(K(t.path, RIFF_SYNC)) },
    { key: "readonly", label: "Không hỗ trợ sửa", test: (t) => t.readonly },
  ];

  filterCtx(): FilterCtx {
    const m = new Map<string, { a: Set<string>; aa: Set<string> }>();
    for (const t of this.tracks) {
      if (!this.editable(t)) continue;
      const s = m.get(t.dir) ?? { a: new Set(), aa: new Set() };
      s.a.add(this.val(t, "album"));
      s.aa.add(this.val(t, "albumartist"));
      m.set(t.dir, s);
    }
    return { inconsistent: new Set([...m].filter(([, s]) => s.a.size > 1 || s.aa.size > 1).map(([d]) => d)) };
  }

  visible(): TrackRow[] {
    if (this.visibleCache?.v === this.version) return this.visibleCache.rows;
    const ctx = this.quick === "incons" ? this.filterCtx() : { inconsistent: new Set<string>() };
    const f = this.quickFilters.find((x) => x.key === this.quick) ?? this.quickFilters[0];
    const q = fold(this.query.trim());
    const folder = this.folder ? this.folder.replace(/\/$/, "") : null;
    let rows = this.tracks.filter(
      (t) =>
        (!folder || t.dir === folder || t.dir.startsWith(folder + "/")) &&
        f.test(t, ctx) &&
        (!q || ["title", "artist", "album", "albumartist"].some((k) => fold(this.val(t, k)).includes(q)) || fold(t.file).includes(q)),
    );
    if (this.sort) {
      const { field, desc } = this.sort;
      const num = ["track", "year", "disc", "durationMs"].includes(field);
      const key = (t: TrackRow): string | number =>
        field === "file" ? t.file : field === "format" ? t.format : field === "durationMs" ? t.durationMs ?? 0 : num ? +this.val(t, field) || 0 : this.val(t, field);
      rows = [...rows].sort((a, b) => {
        const x = key(a), y = key(b);
        const c = typeof x === "number" && typeof y === "number" ? x - y : String(x).localeCompare(String(y), "vi");
        return desc ? -c : c;
      });
    }
    this.visibleCache = { v: this.version, rows };
    return rows;
  }
}

export const store = new Store();

export function useStore<T>(sel: (s: Store) => T): T {
  useSyncExternalStore(store.subscribe, () => store.version);
  return sel(store);
}
