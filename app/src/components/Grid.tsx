// Track table (APP-LIB-BROWSE, APP-EDIT-INLINE, APP-EDIT-FILL) on Glide Data Grid.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  CompactSelection, DataEditor, GridCellKind, type EditableGridCell, type FillPatternEventArgs, type GridCell,
  type GridColumn, type GridSelection, type Item, type Rectangle, type Theme, type DataEditorRef,
} from "@glideapps/glide-data-grid";
import "@glideapps/glide-data-grid/dist/index.css";
import { store, useStore, K } from "../store";
import { formatDuration, NUMERIC, validate } from "../fields";
import type { StageInput, TrackRow } from "../api";

type Col = { key: string; title: string; width: number; editable: boolean };
const COLS: Col[] = [
  { key: "_state", title: "", width: 28, editable: false },
  { key: "file", title: "Tên file", width: 260, editable: false },
  { key: "track", title: "#", width: 44, editable: true },
  { key: "title", title: "Title", width: 220, editable: true },
  { key: "artist", title: "Artist", width: 170, editable: true },
  { key: "album", title: "Album", width: 210, editable: true },
  { key: "albumartist", title: "Album Artist", width: 150, editable: true },
  { key: "year", title: "Year", width: 56, editable: true },
  { key: "genre", title: "Genre", width: 110, editable: true },
  { key: "composer", title: "Composer", width: 140, editable: true },
  { key: "disc", title: "Disc", width: 48, editable: true },
  { key: "_folder", title: "Thư mục", width: 160, editable: false },
  { key: "format", title: "Định dạng", width: 104, editable: false },
  { key: "durationMs", title: "T.lượng", width: 64, editable: false },
];

function cssVar(name: string) {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

function makeTheme(): Partial<Theme> {
  return {
    accentColor: cssVar("--accent"),
    accentLight: cssVar("--range"),
    textDark: cssVar("--fg"),
    textMedium: cssVar("--muted"),
    textLight: cssVar("--muted"),
    textHeader: cssVar("--muted"),
    bgCell: cssVar("--surface"),
    bgCellMedium: cssVar("--surface-2"),
    bgHeader: cssVar("--surface"),
    bgHeaderHasFocus: cssVar("--surface-2"),
    bgHeaderHovered: cssVar("--surface-2"),
    borderColor: cssVar("--line-2"),
    horizontalBorderColor: cssVar("--line-2"),
    fontFamily: cssVar("--f-body"),
    baseFontStyle: "13px",
    headerFontStyle: "600 12px",
    editorFontSize: "13px",
    cellHorizontalPadding: 8,
  };
}

/** Values for the fill destination, one column at a time (APP-STAGE-R16). */
export function fillValue(field: string, src: string[], pos: number, down: boolean, mode: "copy" | "series"): string {
  const n = src.length;
  if (mode === "series") {
    if (NUMERIC.has(field) && src.every((v) => /^\d+$/.test(v))) {
      const st = stepOf(src) ?? 1;
      return String(down ? +src[n - 1] + st * pos : +src[0] - st * pos);
    }
    const base = down ? src[n - 1] : src[0];
    const m = base.match(/^(.*?)(\d+)(\D*)$/);
    if (m) return m[1] + String(Math.max(0, +m[2] + (down ? pos : -pos))) + m[3];
  }
  return down ? src[(pos - 1) % n] : src[n - 1 - ((pos - 1) % n)];
}
function stepOf(src: string[]): number | null {
  if (src.length < 2 || src.some((v) => !/^\d+$/.test(v))) return null;
  const d = +src[1] - +src[0];
  for (let i = 2; i < src.length; i++) if (+src[i] - +src[i - 1] !== d) return null;
  return d;
}

let altDown = false;
window.addEventListener("keydown", (e) => (altDown = e.altKey));
window.addEventListener("keyup", (e) => (altDown = e.altKey));

export default function Grid() {
  const rows = useStore((s) => s.visible());
  useStore((s) => s.version);
  const ref = useRef<DataEditorRef>(null);
  const [theme, setTheme] = useState(makeTheme);
  const [sel, setSel] = useState<GridSelection>({ columns: CompactSelection.empty(), rows: CompactSelection.empty() });

  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const f = () => setTheme(makeTheme());
    mq.addEventListener("change", f);
    return () => mq.removeEventListener("change", f);
  }, []);

  // keep the selection inside the visible rows when filters change
  useEffect(() => {
    setSel({ columns: CompactSelection.empty(), rows: CompactSelection.empty() });
    store.setSelected([]);
  }, [store.folder, store.quick, store.query]);

  const getCellContent = useCallback(
    ([c, r]: Item): GridCell => {
      const t = rows[r];
      const col = COLS[c];
      if (!t) return { kind: GridCellKind.Text, data: "", displayData: "", allowOverlay: false };
      if (col.key === "_state") {
        const st = store.status.get(t.path)?.status;
        const [mark, color] = t.error
          ? ["●", cssVar("--danger")]
          : st === "conflict"
            ? ["◆", cssVar("--warn")]
            : st === "error"
              ? ["●", cssVar("--danger")]
              : store.hasPending(t)
                ? ["●", cssVar("--pending-fg")]
                : t.readonly
                  ? ["–", cssVar("--muted")]
                  : t.riffMismatch
                    ? ["○", cssVar("--muted")]
                    : ["", ""];
        return { kind: GridCellKind.Text, data: mark, displayData: mark, allowOverlay: false, readonly: true, themeOverride: color ? { textDark: color } : undefined };
      }
      if (!col.editable) {
        const d =
          col.key === "file" ? t.file
          : col.key === "_folder" ? t.dir.split("/").pop() ?? ""
          : col.key === "format" ? t.format
          : formatDuration(t.durationMs);
        return { kind: GridCellKind.Text, data: d, displayData: d, allowOverlay: false, readonly: true, themeOverride: { textDark: cssVar("--muted") } };
      }
      const v = store.val(t, col.key);
      const pending = store.isPending(t, col.key);
      const editable = store.editable(t);
      return {
        kind: GridCellKind.Text,
        data: v,
        displayData: v,
        allowOverlay: editable,
        readonly: !editable,
        contentAlign: NUMERIC.has(col.key) ? "right" : undefined,
        themeOverride: pending ? { bgCell: cssVar("--pending"), textDark: cssVar("--pending-fg") } : editable ? undefined : { textDark: cssVar("--muted") },
      };
    },
    [rows, theme, store.version],
  );

  const columns: GridColumn[] = useMemo(
    () => COLS.map((c) => ({ id: c.key, title: c.title + (store.sort?.field === c.key ? (store.sort.desc ? " ↓" : " ↑") : ""), width: c.width })),
    [store.sort?.field, store.sort?.desc],
  );

  // ---- selection -> inspector
  const onSel = (s: GridSelection) => {
    setSel(s);
    const set = new Set<number>();
    s.rows.toArray().forEach((r) => set.add(r));
    const rg = s.current?.range;
    if (rg) for (let r = rg.y; r < rg.y + rg.height; r++) set.add(r);
    s.current?.rangeStack.forEach((x) => { for (let r = x.y; r < x.y + x.height; r++) set.add(r); });
    store.setSelected([...set].sort((a, b) => a - b).map((r) => rows[r]?.path).filter(Boolean));
  };

  // ---- edits (inline, Glide's own paste/downFill)
  const onCellsEdited = (items: readonly { location: Item; value: EditableGridCell }[]) => {
    const inputs: StageInput[] = [];
    let bad = 0;
    for (const { location: [c, r], value } of items) {
      const col = COLS[c];
      const t = rows[r];
      if (!t || !col?.editable || value.kind !== GridCellKind.Text) continue;
      const v = value.data.normalize("NFC").replace(/\s+/g, " ").trim();
      const er = validate(col.key, v) ?? crossCheck(t, col.key, v);
      if (er) { bad++; if (items.length === 1) store.notify(er); continue; }
      inputs.push({ path: t.path, field: col.key, value: v });
    }
    if (inputs.length) {
      const r = store.edit(inputs, items.length > 1 ? "Sửa nhiều ô" : "Sửa ô");
      if (items.length > 1) report("Đã điền", r.applied, r.skipped + r.invalid + bad);
    }
    return true;
  };

  // ---- fill handle (APP-STAGE-R16, R17)
  const doFill = (src: Rectangle, dest: Rectangle, force?: "copy" | "series") => {
    const down = dest.y + dest.height > src.y + src.height || dest.y >= src.y;
    const targets: number[] = [];
    if (down) for (let r = src.y + src.height; r < dest.y + dest.height; r++) targets.push(r);
    else for (let r = src.y - 1; r >= dest.y; r--) targets.push(r);
    const inputs: StageInput[] = [];
    let skipped = 0;
    const modes = new Set<string>();
    for (let c = src.x; c < src.x + src.width; c++) {
      const col = COLS[c];
      if (!col?.editable) continue;
      const vals: string[] = [];
      for (let r = src.y; r < src.y + src.height; r++) vals.push(store.val(rows[r], col.key));
      let mode: "copy" | "series" = force ?? (NUMERIC.has(col.key) && stepOf(vals) != null ? "series" : "copy");
      if (!force && altDown) mode = mode === "series" ? "copy" : "series";
      modes.add(mode);
      targets.forEach((r, i) => {
        const t = rows[r];
        if (!t || !store.editable(t)) { skipped++; return; }
        const v = fillValue(col.key, vals, i + 1, down, mode);
        if (validate(col.key, v) || crossCheck(t, col.key, v)) { skipped++; return; }
        inputs.push({ path: t.path, field: col.key, value: v });
      });
    }
    const r = store.edit(inputs, "Kéo điền");
    const op = store.lastOp();
    if (r.applied && op) {
      store.lastFill = { op, mode: modes.has("series") ? "series" : "copy", redo: (m) => { if (store.undoSilently(op)) doFill(src, dest, m); } };
    }
    report("Đã điền", inputs.length, skipped + r.skipped);
  };
  const onFillPattern = (e: FillPatternEventArgs) => {
    e.preventDefault();
    doFill(e.patternSource, e.fillDestination);
  };

  // ---- paste (APP-STAGE-R18)
  const onPaste = (target: Item, values: readonly (readonly string[])[]) => {
    const rg = sel.current?.range;
    const inputs: StageInput[] = [];
    let skipped = 0, extraRows = 0;
    const put = (r: number, c: number, raw: string) => {
      const t = rows[r], col = COLS[c];
      if (!t || !col?.editable || !store.editable(t)) { skipped++; return; }
      const v = raw.normalize("NFC").replace(/\s+/g, " ").trim();
      if (validate(col.key, v)) { skipped++; return; }
      inputs.push({ path: t.path, field: col.key, value: v });
    };
    if (values.length === 1 && values[0].length === 1 && rg && rg.width * rg.height > 1) {
      for (let r = rg.y; r < rg.y + rg.height; r++) for (let c = rg.x; c < rg.x + rg.width; c++) put(r, c, values[0][0]);
    } else {
      values.forEach((line, i) => {
        const r = target[1] + i;
        if (r >= rows.length) { extraRows++; return; }
        line.forEach((v, j) => (target[0] + j < COLS.length ? put(r, target[0] + j, v) : skipped++));
      });
    }
    const res = store.edit(inputs, "Dán");
    store.notify(`Đã dán ${inputs.length - res.skipped - res.invalid} ô` + (skipped + res.skipped ? `, bỏ qua ${skipped + res.skipped} ô` : "") + (extraRows ? `, bỏ ${extraRows} dòng thừa` : ""));
    return false;
  };

  // ---- delete (APP-STAGE-R19)
  const onDelete = (s: GridSelection) => {
    const rg = s.current?.range;
    if (!rg) return false;
    const inputs: StageInput[] = [];
    for (let r = rg.y; r < rg.y + rg.height; r++)
      for (let c = rg.x; c < rg.x + rg.width; c++) if (COLS[c]?.editable && rows[r]) inputs.push({ path: rows[r].path, field: COLS[c].key, value: "" });
    const res = store.edit(inputs, "Xoá giá trị");
    store.notify(`Xoá giá trị: ${res.applied} ô`);
    return false;
  };

  // Our own paste listener: Glide reads navigator.clipboard, which WebKit gates behind a
  // "Paste" prompt. The paste event already carries the clipboard text (APP-STAGE-R18).
  const wrapRef = useRef<HTMLDivElement>(null);
  const pasteRef = useRef(onPaste);
  pasteRef.current = onPaste;
  const selRef = useRef(sel);
  selRef.current = sel;
  useEffect(() => {
    const f = (e: ClipboardEvent) => {
      if (!wrapRef.current?.contains(document.activeElement)) return;
      if (document.querySelector("#portal textarea, #portal input")) return; // cell editor open: normal text paste
      const rg = selRef.current.current?.range;
      if (!rg) return;
      let text = e.clipboardData?.getData("text/plain") ?? "";
      e.preventDefault();
      text = text.replace(/\r\n?/g, "\n");
      if (text.endsWith("\n")) text = text.slice(0, -1);
      pasteRef.current([rg.x, rg.y], text.split("\n").map((l) => l.split("\t")));
    };
    window.addEventListener("paste", f, true);
    return () => window.removeEventListener("paste", f, true);
  }, []);

  const onHeaderClicked = (c: number) => {
    const key = COLS[c].key;
    if (key === "_state") return;
    const cur = store.sort;
    store.setView({ sort: cur?.field === key ? (cur.desc ? null : { field: key, desc: true }) : { field: key, desc: false } });
  };

  useEffect(() => {
    if (import.meta.env.DEV) (window as any).__grid = ref.current;
  });

  if (!rows.length) {
    return (
      <div className="empty">
        <p>{store.tracks.length ? "Không có track nào khớp" : "Chưa có track nào"}</p>
        {store.tracks.length > 0 && (
          <button className="btn" onClick={() => store.setView({ quick: "all", query: "", folder: null })}>Xoá bộ lọc</button>
        )}
      </div>
    );
  }

  return (
    <div ref={wrapRef} style={{ height: "100%" }}>
    <DataEditor
      ref={ref}
      width="100%"
      height="100%"
      theme={theme}
      columns={columns}
      rows={rows.length}
      rowHeight={28}
      headerHeight={30}
      getCellContent={getCellContent}
      onCellsEdited={onCellsEdited}
      gridSelection={sel}
      onGridSelectionChange={onSel}
      rowMarkers="clickable-number"
      rangeSelect="multi-rect"
      fillHandle={true}
      allowedFillDirections="vertical"
      onFillPattern={onFillPattern}
      onDelete={onDelete}
      onHeaderClicked={onHeaderClicked}
      keybindings={{ downFill: true, selectAll: true, copy: true, paste: false, search: false }}
      smoothScrollY
      freezeColumns={2}
      getCellsForSelection={true}
    />
    </div>
  );
}

function crossCheck(t: TrackRow, field: string, v: string): string | null {
  const tr = field === "track" ? v : store.val(t, "track");
  const tt = field === "tracktotal" ? v : store.val(t, "tracktotal");
  if (tr && tt && +tr > +tt) return "Track không được lớn hơn Track Total";
  return null;
}

function report(prefix: string, n: number, skipped: number) {
  store.notify(n ? `${prefix} ${n} ô${skipped ? `, bỏ qua ${skipped} ô` : ""}` : "Không có ô nào được điền");
}

export { K };
