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
  /** background re-check of one folder (APP-SCAN-R12): no progress bar */
  checking: string | null = null;
  checkedAt = new Map<string, number>();
  status = new Map<string, { status: string; message: string | null }>();
  toast: { text: string; id: number } | null = null;
  lastFill: { op: Op; redo: (mode: "copy" | "series") => void; mode: "copy" | "series" } | null = null;
  private listeners = new Set<() => void>();
  // ---- per-row caches so a library of 100k+ tracks stays responsive: results are
  // recomputed only for rows whose data or staged values changed (rowRev)
  private rowRev = new Map<string, number>();
  private dirRev = new Map<string, number>();
  private epoch = 0; // bumps on changes that affect every row (sources, status)
  private pendingCount = new Map<string, number>();
  private flagCache = new Map<string, { rev: number; epoch: number; bits: number }>();
  private searchCache = new Map<string, { rev: number; text: string }>();
  private dirCache = new Map<string, { rev: number; epoch: number; v: boolean }>();
  private dirRows = new Map<string, TrackRow[]>();
  private pendingMerge: { rows: TrackRow[]; removed: string[] } = { rows: [], removed: [] };
  private mergeTimer: ReturnType<typeof setTimeout> | null = null;
  private countsCache: { v: number; counts: Map<string, number> } | null = null;
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
    try {
      // reopen where the user left off, if that folder is still in the library
      const v = JSON.parse(localStorage.getItem("mlt-view") || "{}");
      if (v.folder && this.tracks.some((t) => t.dir === v.folder || t.dir.startsWith(v.folder + "/"))) this.folder = v.folder;
      if (v.quick && this.quickFilters.some((f) => f.key === v.quick)) this.quick = v.quick;
    } catch {
      /* ignore */
    }
    this.emit();
  }
  setTracks(rows: TrackRow[]) {
    this.tracks = rows;
    this.byPath = new Map(rows.map((r) => [r.path, r]));
    this.rebuildDirs();
    this.invalidateAll();
  }
  setStaged(list: StagedChange[]) {
    this.staged = new Map(list.map((s) => [K(s.path, s.field), { orig: s.orig, value: s.value }]));
    this.pendingCount = new Map();
    for (const s of list) this.pendingCount.set(s.path, (this.pendingCount.get(s.path) ?? 0) + 1);
    this.invalidateAll();
  }
  /** Scan batches arrive every ~200 files: merge them at most twice a second (one sort per flush). */
  mergeRows(rows: TrackRow[], removed: string[]) {
    this.pendingMerge.rows.push(...rows);
    this.pendingMerge.removed.push(...removed);
    if (!this.mergeTimer) this.mergeTimer = setTimeout(() => this.flushMerge(), 500);
  }
  flushMerge() {
    if (this.mergeTimer) clearTimeout(this.mergeTimer);
    this.mergeTimer = null;
    const { rows, removed } = this.pendingMerge;
    this.pendingMerge = { rows: [], removed: [] };
    if (!rows.length && !removed.length) return;
    for (const r of rows) { this.byPath.set(r.path, r); this.bump(r.path, r.dir); }
    for (const p of removed) { const d = this.byPath.get(p)?.dir; this.byPath.delete(p); this.bump(p, d); }
    this.tracks = [...this.byPath.values()].sort((a, b) => (a.dir === b.dir ? (a.file < b.file ? -1 : a.file > b.file ? 1 : 0) : a.dir < b.dir ? -1 : 1));
    this.rebuildDirs();
    this.emit();
  }
  async refreshSources() {
    this.sources = await api.listSources();
    this.epoch++;
    this.emit();
  }
  /** Row data or staged values of `path` changed. */
  bump(path: string, dir?: string) {
    this.rowRev.set(path, (this.rowRev.get(path) ?? 0) + 1);
    const d = dir ?? this.byPath.get(path)?.dir;
    if (d) this.dirRev.set(d, (this.dirRev.get(d) ?? 0) + 1);
  }
  invalidateAll() {
    this.epoch++;
    this.flagCache.clear();
    this.searchCache.clear();
    this.dirCache.clear();
  }
  private rebuildDirs() {
    this.dirRows = new Map();
    for (const t of this.tracks) {
      const a = this.dirRows.get(t.dir);
      if (a) a.push(t); else this.dirRows.set(t.dir, [t]);
    }
  }
  private setStagedValue(key: string, path: string, value: { orig: string; value: string } | undefined) {
    const had = this.staged.has(key);
    if (value === undefined) this.staged.delete(key); else this.staged.set(key, value);
    const has = value !== undefined;
    if (had !== has) {
      const n = (this.pendingCount.get(path) ?? 0) + (has ? 1 : -1);
      if (n > 0) this.pendingCount.set(path, n); else this.pendingCount.delete(path);
    }
    this.bump(path);
  }

  // ---------------------------------------------------------------- values
  val(t: TrackRow, field: string): string {
    return this.staged.get(K(t.path, field))?.value ?? t.fields[field] ?? "";
  }
  isPending(t: TrackRow, field: string) {
    return this.staged.has(K(t.path, field));
  }
  hasPending(t: TrackRow) {
    return this.pendingCount.has(t.path);
  }
  editable(t: TrackRow) {
    const src = this.sources.find((s) => s.id === t.sourceId);
    return !t.error && !t.readonly && src?.status !== "unavailable";
  }
  pendingFiles() {
    return this.pendingCount.size;
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
      this.setStagedValue(c.key, c.path, v === undefined ? undefined : { orig, value: v });
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
      pseudo.forEach((k) => this.setStagedValue(k, k.split("\u0001")[0], undefined));
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
    try {
      localStorage.setItem("mlt-view", JSON.stringify({ folder: this.folder, quick: this.quick }));
    } catch {
      /* remembering the view is a convenience only */
    }
    this.emit();
  }
  notify(text: string) {
    this.toast = { text, id: Date.now() };
    this.emit();
  }

  /** Row tests whose result depends only on the row and its staged values: cached per row. */
  private rowTests: [string, (t: TrackRow) => boolean][] = [
    ["pending", (t) => this.hasPending(t)],
    ["noaa", (t) => this.editable(t) && !this.val(t, "albumartist")],
    ["noalbum", (t) => this.editable(t) && !this.val(t, "album")],
    ["notrack", (t) => this.editable(t) && !this.val(t, "track")],
    ["noart", (t) => this.editable(t) && !t.hasArt],
    ["placeholder", (t) => this.editable(t) && ["artist", "album", "title", "albumartist"].some((f) => PLACEHOLDERS.test(this.val(t, f)))],
    ["space", (t) => this.editable(t) && TEXT_FIELDS.some((f) => { const v = this.val(t, f); return v !== v.replace(/\s+/g, " ").trim(); })],
    ["nfd", (t) => this.editable(t) && TEXT_FIELDS.some((f) => { const v = this.val(t, f); return v !== v.normalize("NFC"); })],
    ["aa", (t) => t.aaMismatch && !this.staged.has(K(t.path, AA_SYNC)) && !this.isPending(t, "albumartist")],
    ["riff", (t) => t.riffMismatch && !this.staged.has(K(t.path, RIFF_SYNC))],
  ];
  private bitOf = new Map(this.rowTests.map(([k], i) => [k, 1 << i]));
  private bits(t: TrackRow): number {
    const rev = this.rowRev.get(t.path) ?? 0;
    const c = this.flagCache.get(t.path);
    if (c && c.rev === rev && c.epoch === this.epoch) return c.bits;
    let b = 0;
    this.rowTests.forEach(([, test], i) => { if (test(t)) b |= 1 << i; });
    this.flagCache.set(t.path, { rev, epoch: this.epoch, bits: b });
    return b;
  }
  private flag = (key: string) => (t: TrackRow) => (this.bits(t) & this.bitOf.get(key)!) !== 0;
  /** APP-LIB-FILTER "Album không đồng nhất", cached per folder. */
  dirInconsistent(dir: string): boolean {
    const rev = this.dirRev.get(dir) ?? 0;
    const c = this.dirCache.get(dir);
    if (c && c.rev === rev && c.epoch === this.epoch) return c.v;
    const a = new Set<string>(), aa = new Set<string>();
    for (const t of this.dirRows.get(dir) ?? []) {
      if (!this.editable(t)) continue;
      a.add(this.val(t, "album"));
      aa.add(this.val(t, "albumartist"));
    }
    const v = a.size > 1 || aa.size > 1;
    this.dirCache.set(dir, { rev, epoch: this.epoch, v });
    return v;
  }
  private searchText(t: TrackRow): string {
    const rev = this.rowRev.get(t.path) ?? 0;
    const c = this.searchCache.get(t.path);
    if (c && c.rev === rev) return c.text;
    const text = [fold(this.val(t, "title")), fold(this.val(t, "artist")), fold(this.val(t, "album")), fold(this.val(t, "albumartist")), fold(t.file)].join("\u0001");
    this.searchCache.set(t.path, { rev, text });
    return text;
  }

  quickFilters: QuickFilter[] = [
    { key: "all", label: "Tất cả", test: () => true },
    { key: "pending", label: "Có thay đổi chờ", test: this.flag("pending") },
    { key: "err", label: "Lỗi", test: (t) => !!t.error || this.status.has(t.path) },
    { key: "noaa", label: "Thiếu Album Artist", test: this.flag("noaa") },
    { key: "noalbum", label: "Thiếu Album", test: this.flag("noalbum") },
    { key: "notrack", label: "Thiếu Track", test: this.flag("notrack") },
    { key: "noart", label: "Thiếu ảnh bìa", test: this.flag("noart") },
    { key: "incons", label: "Album không đồng nhất", test: (t) => this.editable(t) && this.dirInconsistent(t.dir) },
    { key: "placeholder", label: "Giá trị giữ chỗ", test: this.flag("placeholder") },
    { key: "space", label: "Có khoảng trắng thừa", test: this.flag("space") },
    { key: "nfd", label: "Không chuẩn Unicode", test: this.flag("nfd") },
    { key: "aa", label: "Album Artist lệch giữa các khoá", test: this.flag("aa") },
    { key: "riff", label: "WAV: RIFF INFO lệch", test: this.flag("riff") },
    { key: "readonly", label: "Không hỗ trợ sửa", test: (t) => t.readonly },
  ];

  /** Kept for callers that pass a context; folder results come from dirInconsistent. */
  filterCtx(): FilterCtx {
    return { inconsistent: { has: (d: string) => this.dirInconsistent(d) } as unknown as Set<string> };
  }

  /** Counts for every quick filter in one pass (sidebar), cached per store version. */
  counts(): Map<string, number> {
    if (this.countsCache?.v === this.version) return this.countsCache.counts;
    const m = new Map<string, number>(this.quickFilters.map((f) => [f.key, 0]));
    const inc = (k: string) => m.set(k, m.get(k)! + 1);
    const keys = this.rowTests.map(([k]) => k);
    for (const t of this.tracks) {
      let b = this.bits(t);
      for (let i = 0; b; i++, b >>= 1) if (b & 1) inc(keys[i]);
      if (t.error || this.status.has(t.path)) inc("err");
      if (t.readonly) inc("readonly");
      if (this.editable(t) && this.dirInconsistent(t.dir)) inc("incons");
    }
    m.set("all", this.tracks.length);
    this.countsCache = { v: this.version, counts: m };
    return m;
  }

  visible(): TrackRow[] {
    if (this.visibleCache?.v === this.version) return this.visibleCache.rows;
    const ctx = this.filterCtx();
    const f = this.quickFilters.find((x) => x.key === this.quick) ?? this.quickFilters[0];
    const q = fold(this.query.trim());
    const folder = this.folder ? this.folder.replace(/\/$/, "") : null;
    let rows = this.tracks.filter(
      (t) =>
        (!folder || t.dir === folder || t.dir.startsWith(folder + "/")) &&
        f.test(t, ctx) &&
        (!q || this.searchText(t).includes(q)),
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
