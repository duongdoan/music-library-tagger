// User actions that span the store and the backend.
import { open } from "@tauri-apps/plugin-dialog";
import { api, on, type Source, type StageInput } from "./api";
import { store, K } from "./store";
import { AA_SYNC, fmtN, RIFF_SYNC, TEXT_FIELDS } from "./fields";

let listening = false;
export async function startListeners() {
  if (listening) return;
  listening = true;
  await on.scanProgress((p) => {
    const src = store.sources.find((s) => s.id === p.sourceId);
    store.scan = { ...p, name: src?.name ?? "" };
    store.emit();
  });
  await on.scanBatch((b) => store.mergeRows(b.rows, b.removed));
}

export async function addSource() {
  const dir = await open({ directory: true, multiple: false, title: "Chọn thư mục nhạc" });
  if (!dir || Array.isArray(dir)) return;
  try {
    const src = await api.addSource(dir);
    await store.refreshSources();
    await rescan(src);
  } catch (e) {
    store.notify(String(e));
  }
}

/** Full scan of a source, or a quiet re-check of one folder (APP-SCAN-R12). */
export async function rescan(src: Source, folder?: string, quiet = false) {
  if (store.scan) return;
  store.scan = { sourceId: src.id, phase: "list", done: 0, total: 0, perSec: 0, name: src.name };
  store.emit();
  try {
    const s = await api.scanSource(src.id, folder);
    if (s.unavailable) store.notify(`Không truy cập được «${src.path}». Kiểm tra kết nối ổ đĩa.`);
    else if (s.cancelled) store.notify(`Đã huỷ quét sau ${fmtN(s.read)} file; các file đã đọc được giữ trong chỉ mục`);
    else if (!quiet || s.read || s.removed)
      store.notify(
        s.read + s.removed === 0 && !quiet
          ? "Không có thay đổi nào"
          : `Quét xong: đọc ${fmtN(s.read)} file · mới ${fmtN(s.added)} · thay đổi ${fmtN(s.changed)} · đã xoá ${fmtN(s.removed)} · lỗi ${fmtN(s.errors)} · bỏ qua ${fmtN(s.skipped)}`,
      );
  } catch (e) {
    store.notify(String(e));
  } finally {
    store.scan = null;
    await store.refreshSources();
  }
}

/** After an index upgrade (rows marked mtime -1) re-read those sources once, one after another. */
export async function refreshStaleSources() {
  const stale = store.sources.filter((s) => store.tracks.some((t) => t.sourceId === s.id && t.mtime === -1));
  if (!stale.length) return;
  store.notify(`Đang cập nhật chỉ mục cho ${stale.length} nguồn sau khi nâng cấp app…`);
  for (const s of stale) await rescan(s, undefined, true);
}

function targetRows() {
  const sel = store.selected.map((p) => store.byPath.get(p)!).filter(Boolean);
  return sel.length ? sel : store.visible();
}

/** APP-STAGE-R11 */
export function numberTracks(start = 1, withTotal = true) {
  const rows = store.visible().filter((t) => store.selected.includes(t.path) && store.editable(t));
  if (!rows.length) return store.notify("Chọn các track cần đánh số trước");
  const groups = new Map<string, typeof rows>();
  rows.forEach((t) => groups.set(t.dir, [...(groups.get(t.dir) ?? []), t]));
  const inputs: StageInput[] = [];
  for (const g of groups.values())
    g.forEach((t, i) => {
      inputs.push({ path: t.path, field: "track", value: String(start + i) });
      if (withTotal) inputs.push({ path: t.path, field: "tracktotal", value: String(start + g.length - 1) });
    });
  store.edit(inputs, "Đánh số track");
  store.notify(`Đánh số ${rows.length} track trong ${groups.size} thư mục`);
}

/** APP-EDIT-CASE: whitespace + Unicode clean-up */
export function cleanText() {
  const inputs: StageInput[] = [];
  for (const t of targetRows()) for (const f of TEXT_FIELDS) inputs.push({ path: t.path, field: f, value: store.val(t, f) });
  const r = store.edit(inputs, "Dọn khoảng trắng & Unicode");
  store.notify(r.applied ? `Đã chuẩn hoá ${r.applied} giá trị` : "Không có giá trị nào cần chuẩn hoá");
}

const CASES: Record<string, (s: string) => string> = {
  title: (s) => s.toLocaleLowerCase("vi").replace(/(^|[\s(\[\-/"'])(\p{L})/gu, (_, a, b) => a + b.toLocaleUpperCase("vi")),
  sentence: (s) => { const l = s.toLocaleLowerCase("vi"); return l.charAt(0).toLocaleUpperCase("vi") + l.slice(1); },
  lower: (s) => s.toLocaleLowerCase("vi"),
  upper: (s) => s.toLocaleUpperCase("vi"),
};
export const CASE_LABEL: Record<string, string> = { title: "Viết Hoa Mỗi Từ", sentence: "Viết hoa chữ đầu", lower: "chữ thường", upper: "CHỮ HOA" };

/** APP-STAGE-R12 */
export function changeCase(kind: string, field: string) {
  const f = CASES[kind];
  const inputs = targetRows().map((t) => ({ path: t.path, field, value: f(store.val(t, field)) }));
  const r = store.edit(inputs, `${CASE_LABEL[kind]} (${field})`);
  store.notify(`Đã đổi ${r.applied} giá trị`);
}

/** APP-EDIT-RIFFSYNC + Album Artist keys (APP-TAG-R13): stage a rewrite for every file whose tag areas/keys disagree */
export function syncTags() {
  const rows = targetRows();
  const riff = rows.filter((t) => t.ext === "wav" && t.riffMismatch && !store.staged.has(K(t.path, RIFF_SYNC)));
  const aa = rows.filter((t) => t.aaMismatch && !store.staged.has(K(t.path, AA_SYNC)) && !store.isPending(t, "albumartist"));
  if (!riff.length && !aa.length) return store.notify("Không có file nào lệch RIFF INFO hoặc khoá Album Artist trong vùng chọn");
  store.edit(
    [...riff.map((t) => ({ path: t.path, field: RIFF_SYNC, value: "1" })), ...aa.map((t) => ({ path: t.path, field: AA_SYNC, value: "1" }))],
    "Đồng bộ tag lệch",
  );
  store.notify([riff.length ? `${fmtN(riff.length)} file WAV (RIFF INFO)` : "", aa.length ? `${fmtN(aa.length)} file (khoá Album Artist)` : ""].filter(Boolean).join(" và ") + " đã vào danh sách chờ đồng bộ");
}
