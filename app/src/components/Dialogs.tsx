// Modal dialogs: find & replace, tag from filename, review & apply, history, settings, confirm.
import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { api, on, type ApplyReport, type FileResult, type RunInfo, type StageInput, type TrackRow } from "../api";
import { store, K } from "../store";
import { AA_SYNC, fmtN, LABEL, normalize, RIFF_SYNC, TEXT_FIELDS } from "../fields";


export function Modal({ title, onClose, children, footer, wide }: { title: string; onClose: () => void; children: ReactNode; footer?: ReactNode; wide?: boolean }) {
  useEffect(() => {
    const f = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", f);
    return () => window.removeEventListener("keydown", f);
  }, [onClose]);
  return (
    <div className="modal-bg" onMouseDown={(e) => e.target === e.currentTarget && onClose()}>
      <div className={"modal" + (wide ? " wide" : "")} role="dialog" aria-modal="true" aria-label={title}>
        <header><h5>{title}</h5><span className="spacer" /><button className="btn" onClick={onClose} aria-label="Đóng">✕</button></header>
        <div className="mbody">{children}</div>
        {footer && <footer>{footer}</footer>}
      </div>
    </div>
  );
}

export function Confirm({ text, ok, onOk, onClose, cancel, onCancel }: { text: string; ok: string; onOk: () => void; onClose: () => void; cancel?: string; onCancel?: () => void }) {
  return (
    <Modal title="Xác nhận" onClose={onClose} footer={<><span className="spacer" /><button className="btn" onClick={() => { onClose(); onCancel?.(); }}>{cancel ?? "Huỷ"}</button><button className="btn primary" onClick={() => { onClose(); onOk(); }}>{ok}</button></>}>
      <p>{text}</p>
    </Modal>
  );
}

const show = (v: string) => (v === "" ? <span className="muted">(trống)</span> : v);
const esc = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

// ------------------------------------------------------------------ find & replace (APP-STAGE-R9)

interface Preset { name: string; find: string; rep: string; field: string; mode: string; cs?: boolean }
const PRESETS: Preset[] = [
  { name: 'Xoá tiền tố số dạng "16 / "', find: "^(?:\\d+\\s*/\\s*)+", rep: "", field: "album", mode: "regex" },
  { name: "Bỏ số thứ tự ở đầu Title", find: "^\\d{1,3}[\\s.\\-_]+", rep: "", field: "title", mode: "regex" },
  { name: 'Đảo "Họ, Tên" thành "Tên Họ"', find: "^([^,]+),\\s*(.+)$", rep: "$2 $1", field: "artist", mode: "regex" },
  { name: "Xoá phần trong ngoặc vuông", find: "\\s*\\[[^\\]]*\\]", rep: "", field: "*", mode: "regex" },
  { name: 'Chuẩn hoá "feat."', find: "\\s*[(\\[]?\\s*(?:ft\\.?|feat\\.?|featuring)\\s+([^)\\]]+)[)\\]]?", rep: " (feat. $1)", field: "title", mode: "regex" },
];
function userPresets(): Preset[] {
  try { return JSON.parse(localStorage.getItem("mlt-rx") || "[]"); } catch { return []; }
}

export function ReplaceDialog({ onClose }: { onClose: () => void }) {
  const [find, setFind] = useState("");
  const [rep, setRep] = useState("");
  const [field, setField] = useState("album");
  const [mode, setMode] = useState("contains");
  const [cs, setCs] = useState(false);
  const [scope, setScope] = useState(store.selected.length > 1 ? "sel" : "vis");
  const [presets, setPresets] = useState<Preset[]>([...PRESETS, ...userPresets()]);
  const [off, setOff] = useState<Set<number>>(new Set());

  const { matches, error } = useMemo(() => {
    if (!find) return { matches: [] as { t: TrackRow; k: string; v: string; nv: string }[], error: null };
    let re: RegExp;
    try {
      const fl = cs ? "" : "i";
      const e = esc(find);
      re = mode === "regex" ? new RegExp(find, "g" + fl) : mode === "start" ? new RegExp("^" + e, fl) : mode === "end" ? new RegExp(e + "$", fl) : mode === "whole" ? new RegExp("^" + e + "$", fl) : new RegExp(e, "g" + fl);
    } catch (err) {
      return { matches: [], error: "Biểu thức không hợp lệ: " + (err as Error).message };
    }
    const rows = (scope === "sel" ? store.selected.map((p) => store.byPath.get(p)!).filter(Boolean) : store.visible()).filter((t) => store.editable(t));
    const fields = field === "*" ? TEXT_FIELDS : [field];
    const out: { t: TrackRow; k: string; v: string; nv: string }[] = [];
    for (const t of rows)
      for (const k of fields) {
        const v = store.val(t, k);
        re.lastIndex = 0;
        if (!re.test(v)) continue;
        re.lastIndex = 0;
        const nv = mode === "regex" ? v.replace(re, rep) : v.replace(re, () => rep);
        if (nv !== v) out.push({ t, k, v, nv });
      }
    return { matches: out, error: null };
  }, [find, rep, field, mode, cs, scope]);
  useEffect(() => setOff(new Set()), [matches]);

  const go = () => {
    const inputs: StageInput[] = matches.filter((_, i) => !off.has(i)).map((m) => ({ path: m.t.path, field: m.k, value: m.nv }));
    const r = store.edit(inputs, "Tìm & thay");
    store.notify(`Đã tạo ${r.applied} thay đổi chờ`);
    onClose();
  };
  const count = matches.length - off.size;
  return (
    <Modal title="Tìm & thay" wide onClose={onClose}
      footer={<><span className="muted">{matches.length ? `${fmtN(matches.length)} mục khớp` : ""}</span><span className="spacer" /><button className="btn" onClick={onClose}>Huỷ</button><button className="btn primary" disabled={!count || !!error} onClick={go}>Thay {fmtN(count)} mục</button></>}>
      <div className="row wrap">
        <div className="fld grow"><label htmlFor="rPreset">Mẫu đã lưu</label>
          <select id="rPreset" defaultValue="" onChange={(e) => { const p = presets[+e.target.value]; if (!p) return; setFind(p.find); setRep(p.rep); setField(p.field); setMode(p.mode); setCs(!!p.cs); setScope("vis"); }}>
            <option value="">(chọn mẫu để điền sẵn)</option>
            {presets.map((p, i) => <option key={i} value={i}>{p.name}</option>)}
          </select>
        </div>
        <button className="btn" onClick={() => {
          if (!find) return store.notify("Nhập chuỗi tìm trước khi lưu mẫu");
          const p = { name: `${find} → ${rep || "(xoá)"}`, find, rep, field, mode, cs };
          const u = [...userPresets(), p];
          try { localStorage.setItem("mlt-rx", JSON.stringify(u)); } catch { /* per-device convenience only */ }
          setPresets([...PRESETS, ...u]);
          store.notify(`Đã lưu mẫu "${p.name}"`);
        }}>Lưu mẫu</button>
      </div>
      <div className="form-grid">
        <div className="fld wide"><label htmlFor="rFind">Tìm</label><input id="rFind" autoFocus value={find} onChange={(e) => setFind(e.target.value)} className="mono" /></div>
        <div className="fld wide"><label htmlFor="rRep">Thay bằng</label><input id="rRep" value={rep} placeholder="(chuỗi rỗng = xoá)" onChange={(e) => setRep(e.target.value)} className="mono" />
          {mode === "regex" && <span className="hint-sm">$1, $2… nhóm bắt · $&lt;ten&gt; nhóm đặt tên · $&amp; cả đoạn khớp · $$ ký tự $</span>}</div>
        <div className="fld"><label htmlFor="rField">Trường</label>
          <select id="rField" value={field} onChange={(e) => setField(e.target.value)}>
            {TEXT_FIELDS.map((f) => <option key={f} value={f}>{LABEL[f]}</option>)}<option value="*">Mọi trường văn bản</option>
          </select></div>
        <div className="fld"><label htmlFor="rMode">Kiểu khớp</label>
          <select id="rMode" value={mode} onChange={(e) => setMode(e.target.value)}>
            <option value="contains">Chứa</option><option value="start">Bắt đầu bằng</option><option value="end">Kết thúc bằng</option><option value="whole">Toàn bộ giá trị</option><option value="regex">Biểu thức chính quy</option>
          </select></div>
        <div className="fld"><label htmlFor="rScope">Phạm vi</label>
          <select id="rScope" value={scope} onChange={(e) => setScope(e.target.value)}>
            <option value="vis">Dòng đang hiển thị ({fmtN(store.visible().length)})</option><option value="sel">Dòng đang chọn ({fmtN(store.selected.length)})</option>
          </select></div>
        <label className="checkline"><input type="checkbox" checked={cs} onChange={(e) => setCs(e.target.checked)} />Phân biệt hoa/thường</label>
      </div>
      {error && <p className="err">{error}</p>}
      {find && !error && !matches.length && <p className="err">Không có file nào khớp</p>}
      {matches.length > 0 && (
        <table className="diff"><thead><tr><th /><th>File</th><th>Trường</th><th>Cũ</th><th>Mới</th></tr></thead>
          <tbody>{matches.slice(0, 200).map((m, i) => (
            <tr key={i}><td><input type="checkbox" checked={!off.has(i)} onChange={(e) => setOff((o) => { const n = new Set(o); e.target.checked ? n.delete(i) : n.add(i); return n; })} aria-label="Chọn" /></td>
              <td className="mono small">{m.t.file}</td><td>{LABEL[m.k]}</td><td className="old">{show(m.v)}</td><td><span className="new">{show(m.nv)}</span></td></tr>
          ))}</tbody></table>
      )}
    </Modal>
  );
}

// ------------------------------------------------------------------ tag from filename (APP-STAGE-R10)

const TOKENS = ["title", "artist", "album", "albumartist", "track", "disc", "year", "genre", "composer", "ignore"];

export function PatternDialog({ onClose }: { onClose: () => void }) {
  const [pat, setPat] = useState(() => localStorage.getItem("mlt-pattern") ?? "%track% - %artist% - %title%");
  const rows = useMemo(() => (store.selected.length ? store.selected.map((p) => store.byPath.get(p)!).filter(Boolean) : store.visible()).filter((t) => store.editable(t)), []);
  const { res, toks, error } = useMemo(() => {
    const toks: string[] = [];
    const src = pat.split(/(%\w+%)/).map((p) => {
      const m = p.match(/^%(\w+)%$/);
      if (m && TOKENS.includes(m[1])) { toks.push(m[1]); return ["track", "disc", "year"].includes(m[1]) ? "(\\d+)" : "(.+?)"; }
      return esc(p);
    }).join("");
    if (!toks.length) return { res: [], toks, error: "Mẫu phải chứa ít nhất một trường, ví dụ %title%" };
    const re = new RegExp("^" + src + "$");
    const res = rows.slice(0, 5000).map((t) => {
      const base = pat.includes("/") ? t.path.slice(t.path.lastIndexOf("/", t.path.lastIndexOf("/") - 1) + 1).replace(/\.[^.]+$/, "") : t.file.replace(/\.[^.]+$/, "");
      const m = base.match(re);
      if (!m) return { t, ok: false as const };
      const o: Record<string, string> = {};
      toks.forEach((k, i) => { if (k !== "ignore") o[k] = ["track", "disc"].includes(k) ? String(+m[i + 1]) : normalize(m[i + 1]); });
      return { t, ok: true as const, o };
    });
    return { res, toks, error: null };
  }, [pat, rows]);
  const shown = toks.filter((k) => k !== "ignore");
  const okN = res.filter((r) => r.ok).length;
  const go = () => {
    try { localStorage.setItem("mlt-pattern", pat); } catch { /* optional */ }
    const inputs: StageInput[] = [];
    res.forEach((r) => { if (r.ok) for (const k in r.o) inputs.push({ path: r.t.path, field: k, value: r.o[k] }); });
    const x = store.edit(inputs, "Tag từ tên file");
    store.notify(`Đã tạo ${x.applied} thay đổi chờ`);
    onClose();
  };
  return (
    <Modal title="Tag từ tên file" wide onClose={onClose}
      footer={<><span className="muted">{fmtN(okN)} khớp · {fmtN(res.length - okN)} không khớp</span><span className="spacer" /><button className="btn" onClick={onClose}>Huỷ</button><button className="btn primary" disabled={!okN} onClick={go}>Áp dụng</button></>}>
      <div className="fld"><label htmlFor="pPat">Mẫu (áp lên tên file, bỏ phần đuôi; có "/" thì áp lên thư mục/tên file)</label>
        <input id="pPat" className="mono" value={pat} onChange={(e) => setPat(e.target.value)} autoFocus /></div>
      <div className="chips">{TOKENS.map((t) => <button key={t} className="chip" onClick={() => setPat((p) => p + `%${t}%`)}>%{t}%</button>)}</div>
      {error && <p className="err">{error}</p>}
      <table className="diff"><thead><tr><th>Tên file</th>{shown.map((k) => <th key={k}>{LABEL[k] ?? k}</th>)}</tr></thead>
        <tbody>{res.slice(0, 200).map((r, i) => (
          <tr key={i}><td className="mono small">{r.t.file}</td>
            {r.ok ? shown.map((k) => <td key={k}>{store.val(r.t, k) === r.o[k] ? r.o[k] : <span className="new">{r.o[k]}</span>}</td>)
              : <td colSpan={shown.length} className="err">Không khớp</td>}</tr>
        ))}</tbody></table>
    </Modal>
  );
}

// ------------------------------------------------------------------ review & apply (APP-APPLY-*)

export function ReviewDialog({ onClose, undoOf }: { onClose: () => void; undoOf?: number }) {
  const list = useMemo(() => [...store.staged].map(([key, s]) => {
    const [path, field] = key.split("\u0001");
    return { key, path, field, orig: s.orig, value: s.value, t: store.byPath.get(path)! };
  }).filter((x) => x.t).sort((a, b) => (a.path + a.field).localeCompare(b.path + b.field)), []);
  const [on_, setOn] = useState<Set<string>>(() => new Set(list.map((x) => x.key)));
  const [tab, setTab] = useState<"album" | "file" | "field">("album");
  const [running, setRunning] = useState<{ done: number; total: number; started: number; current: string | null } | null>(null);
  const [results, setResults] = useState<FileResult[]>([]);
  const [report, setReport] = useState<ApplyReport | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const toggle = (keys: string[], v: boolean) => setOn((o) => { const n = new Set(o); keys.forEach((k) => (v ? n.add(k) : n.delete(k))); return n; });
  const files = new Set(list.filter((x) => on_.has(x.key)).map((x) => x.path)).size;
  const dirs = new Set(list.filter((x) => on_.has(x.key)).map((x) => x.t.dir)).size;

  const run = async (inPlace: boolean) => {
    const items = list.filter((x) => on_.has(x.key)).map((x) => ({ path: x.path, field: x.field }));
    setRunning({ done: 0, total: new Set(items.map((i) => i.path)).size, started: Date.now(), current: null });
    const un = await on.applyProgress((p) => {
      if (p.result.status === "writing") {
        setRunning((r) => r && { ...r, current: p.result.path });
        return;
      }
      setRunning((r) => r && { ...r, done: p.done, total: p.total, current: null });
      setResults((r) => [...r, p.result]);
    });
    try {
      const rep = await api.applyRun(items, undoOf, inPlace);
      setReport(rep);
      for (const r of rep.results) {
        if (r.status === "ok") store.status.delete(r.path);
        else if (r.status !== "cancelled") store.status.set(r.path, { status: r.status, message: r.message });
      }
      store.mergeRows(rep.results.filter((r) => r.row && r.status === "ok").map((r) => r.row!), []);
      store.flushMerge();
      store.setStaged(await api.stageList());
      store.undoStack = [];
      store.redoStack = [];
      store.emit();
    } catch (e) {
      setFailure(String(e));
    } finally {
      un();
      setRunning(null);
    }
  };

  if (report || failure)
    return (
      <Modal title="Kết quả áp dụng" wide onClose={onClose}
        footer={<><span className="spacer" />{report && report.ok > 0 && <button className="btn" onClick={() => { onClose(); undoRun(report.runId); }}>Hoàn tác lượt này</button>}<button className="btn primary" onClick={onClose}>Đóng</button></>}>
        {failure && <div className="errbox">{failure}</div>}
        {report && (
          <>
            <div className="stats">
              <div><b className="ok">{fmtN(report.ok)}</b><span>thành công</span></div>
              <div><b className="warn">{fmtN(report.conflict)}</b><span>xung đột</span></div>
              <div><b className="danger">{fmtN(report.error)}</b><span>lỗi</span></div>
              <div><b>{fmtN(report.cancelled)}</b><span>đã huỷ (còn chờ)</span></div>
            </div>
            <p>{report.conflict + report.error + report.cancelled === 0 ? `Đã ghi ${fmtN(list.filter((x) => on_.has(x.key)).length)} thay đổi vào ${fmtN(report.ok)} file.` : `Đã ghi ${fmtN(report.ok)} / ${fmtN(report.results.length)} file.` + (report.ok ? " Giá trị gốc đã lưu trong lịch sử." : " Các thay đổi chưa ghi vẫn ở trạng thái chờ.")}</p>
            {report.results.filter((r) => r.status === "conflict" || r.status === "error").length > 0 && (
              <table className="diff"><thead><tr><th>File</th><th>Lý do</th></tr></thead>
                <tbody>{report.results.filter((r) => r.status === "conflict" || r.status === "error").map((r) => (
                  <tr key={r.path}><td className="mono small">{r.path.split("/").pop()}</td><td>{r.message}</td></tr>
                ))}</tbody></table>
            )}
          </>
        )}
      </Modal>
    );

  return (
    <Modal title={undoOf ? `Xem trước hoàn tác lượt ${undoOf}` : "Xem trước thay đổi"} wide onClose={running ? () => {} : onClose}
      footer={running ? (
        <div className="progress grow"><span>{applyStatus(running)}</span><div className="bar"><i style={{ width: `${(running.done / Math.max(1, running.total)) * 100}%` }} /></div><button className="btn" onClick={() => api.cancelApply()}>Huỷ</button></div>
      ) : (
        <><span><b>{fmtN(on_.size)}</b> thay đổi trên <b>{fmtN(files)}</b> file · {fmtN(dirs)} thư mục</span><span className="spacer" /><button className="btn" onClick={onClose}>Đóng</button><button className="btn" disabled={!on_.size} onClick={() => run(true)}
          title="Ghi thẳng vào file như bộ script cũ: nhanh trên NAS; vẫn có nhật ký để hoàn tác giá trị tag, nhưng mất kết nối đúng lúc ghi có thể làm hỏng file đang ghi">Ghi nhanh (ghi thẳng)</button>
        <button className="btn primary" disabled={!on_.size} onClick={() => run(false)}
          title="Ghi vào bản tạm, kiểm tra rồi mới thay file gốc: file gốc không bao giờ ở trạng thái ghi dở; trên NAS chậm vì phải chép cả file">Ghi an toàn · {fmtN(on_.size)} thay đổi</button></>
      )}>
      {running ? (
        <table className="diff"><tbody>{results.slice(-300).map((r) => (
          <tr key={r.path}><td className="mono small">{r.path.split("/").pop()}</td>
            <td>{r.status === "ok" ? <span className="pill ok">Đã ghi · đã xác minh</span> : r.status === "conflict" ? <span className="pill warn">Xung đột</span> : r.status === "error" ? <span className="pill danger">{r.message}</span> : <span className="pill">Đã huỷ</span>}</td></tr>
        ))}</tbody></table>
      ) : (
        <>
          <div className="tabs">
            <button className={tab === "album" ? "on" : ""} onClick={() => setTab("album")}>Theo album</button>
            <button className={tab === "field" ? "on" : ""} onClick={() => setTab("field")}>Theo trường</button>
            <button className={tab === "file" ? "on" : ""} onClick={() => setTab("file")}>Theo file</button>
          </div>
          {tab === "album" ? <ByAlbum list={list} on_={on_} toggle={toggle} /> : tab === "file" ? <ByFile list={list} on_={on_} toggle={toggle} /> : <ByField list={list} on_={on_} toggle={toggle} />}
        </>
      )}
    </Modal>
  );
}

function fmtSecs(sec: number) {
  if (sec < 60) return `${Math.max(1, Math.round(sec))} giây`;
  const m = Math.floor(sec / 60);
  return m < 60 ? `${m} phút` : `${Math.floor(m / 60)} giờ ${m % 60} phút`;
}

/** "Đang ghi 12 / 626 file · 11,9 giây/file · còn khoảng 2 giờ 3 phút · file đang ghi" */
function applyStatus(r: { done: number; total: number; started: number; current: string | null }) {
  const parts = [`Đang ghi ${fmtN(r.done)} / ${fmtN(r.total)} file`];
  if (r.done > 0) {
    const per = (Date.now() - r.started) / 1000 / r.done;
    parts.push(`${per.toLocaleString("vi-VN", { maximumFractionDigits: 1 })} giây/file`);
    if (r.total > r.done) parts.push(`còn khoảng ${fmtSecs(per * (r.total - r.done))}`);
  }
  if (r.current) parts.push(r.current.split("/").pop()!);
  return parts.join(" · ");
}

/** What a staged row changes, in the file's own terms (pseudo rows spelled out). */
function describe(x: Item): { before: string; after: string } {
  if (x.field === AA_SYNC) {
    const keys = x.t.aaKeys ?? [];
    const id3 = keys.some(([k]) => k === "TPE2" || k.startsWith("TXXX"));
    const before = keys.map(([k, v]) => `${k} = ${v || "(trống)"}`);
    if (!id3 && !keys.some(([k, v]) => k === "ALBUMARTIST" && v)) {
      if (!keys.some(([k]) => k === "ALBUMARTIST")) before.push("ALBUMARTIST: (không có)");
    }
    const targets = id3 ? [...new Set(["TPE2", ...keys.map(([k]) => k)])] : [...new Set(["ALBUMARTIST", ...keys.map(([k]) => k)])];
    const v = x.t.fields.albumartist ?? "";
    return { before: before.join(" · ") || "(khoá lệch)", after: `${targets.join(", ")} = ${v || "(trống)"}` };
  }
  if (x.field === RIFF_SYNC) return { before: "RIFF INFO khác ID3", after: "RIFF INFO = giá trị ID3" };
  return { before: x.orig, after: x.value };
}
const changeId = (x: Item) => { const d = describe(x); return x.field + "\u0000" + d.before + "\u0000" + d.after; };
function Before({ x }: { x: Item }) {
  const d = describe(x);
  return <span className="old">{show(d.before)}</span>;
}
function After({ x }: { x: Item }) {
  const d = describe(x);
  return <span className="new">{show(d.after)}</span>;
}

/** One block per album folder; identical changes inside it collapse to one row with a file count. */
function ByAlbum({ list, on_, toggle }: { list: Item[]; on_: Set<string>; toggle: (k: string[], v: boolean) => void }) {
  const [open, setOpen] = useState<Set<string>>(new Set());
  const [limit, setLimit] = useState(60);
  const albums = useMemo(() => {
    const m = new Map<string, Item[]>();
    list.forEach((x) => m.set(x.t.dir, [...(m.get(x.t.dir) ?? []), x]));
    return [...m.entries()];
  }, [list]);
  const flip = (id: string) => setOpen((o) => { const n = new Set(o); n.has(id) ? n.delete(id) : n.add(id); return n; });
  return (
    <>
      <p className="muted small">{fmtN(albums.length)} album · thay đổi giống nhau trong một album được gộp thành một dòng; bấm số file để xem từng file.</p>
      <table className="diff album-view">
        <thead><tr><th /><th>Trường</th><th>Hiện tại</th><th>Sau khi ghi</th><th className="num">File</th></tr></thead>
        <tbody>
          {albums.slice(0, limit).map(([dir, items]) => {
            const keys = items.map((x) => x.key);
            const files = new Set(items.map((x) => x.path)).size;
            const album = items[0].t.fields.album || "(chưa có Album)";
            const groups = new Map<string, Item[]>();
            items.forEach((x) => { const id = changeId(x); groups.set(id, [...(groups.get(id) ?? []), x]); });
            return [
              <tr className="dir" key={"d" + dir}>
                <td><input type="checkbox" checked={keys.every((k) => on_.has(k))} onChange={(e) => toggle(keys, e.target.checked)} aria-label="Chọn album" /></td>
                <td colSpan={3}><b>{dir.split("/").pop()}</b> <span className="muted">· Album: {album}</span></td>
                <td className="num">{fmtN(files)}</td>
              </tr>,
              ...[...groups.entries()].map(([id, xs]) => {
                const ks = xs.map((x) => x.key);
                const gid = dir + id;
                return [
                  <tr key={gid}>
                    <td><input type="checkbox" checked={ks.every((k) => on_.has(k))} onChange={(e) => toggle(ks, e.target.checked)} aria-label="Chọn nhóm" /></td>
                    <td>{LABEL[xs[0].field]}</td>
                    <td><Before x={xs[0]} /></td>
                    <td><After x={xs[0]} /></td>
                    <td className="num"><button className="link" onClick={() => flip(gid)}>{fmtN(xs.length)} {open.has(gid) ? "▾" : "▸"}</button></td>
                  </tr>,
                  open.has(gid) ? (
                    <tr key={gid + "f"} className="file"><td /><td colSpan={4}>{xs.map((x) => x.t.file).join(" · ")}</td></tr>
                  ) : null,
                ];
              }),
            ];
          })}
        </tbody>
      </table>
      {albums.length > limit && <button className="btn" onClick={() => setLimit((l) => l + 100)}>Hiện thêm ({fmtN(albums.length - limit)} album)</button>}
    </>
  );
}

type Item = { key: string; path: string; field: string; orig: string; value: string; t: TrackRow };

function ByFile({ list, on_, toggle }: { list: Item[]; on_: Set<string>; toggle: (k: string[], v: boolean) => void }) {
  const [limit, setLimit] = useState(400);
  const rows: ReactNode[] = [];
  let dir = "", file = "";
  for (const x of list.slice(0, limit)) {
    if (x.t.dir !== dir) {
      dir = x.t.dir;
      const ks = list.filter((y) => y.t.dir === dir).map((y) => y.key);
      rows.push(<tr className="dir" key={"d" + dir}><td><input type="checkbox" checked={ks.every((k) => on_.has(k))} onChange={(e) => toggle(ks, e.target.checked)} aria-label="Chọn thư mục" /></td><td colSpan={3}>{dir}</td></tr>);
    }
    if (x.path !== file) {
      file = x.path;
      const ks = list.filter((y) => y.path === file).map((y) => y.key);
      const st = store.status.get(file);
      rows.push(<tr className="file" key={"f" + file}><td><input type="checkbox" checked={ks.every((k) => on_.has(k))} onChange={(e) => toggle(ks, e.target.checked)} aria-label="Chọn file" /></td><td colSpan={3}>{x.t.file}{st && <span className="pill warn"> {st.status === "conflict" ? "xung đột trước đó" : "lỗi trước đó"}</span>}</td></tr>);
    }
    rows.push(
      <tr key={x.key}><td><input type="checkbox" checked={on_.has(x.key)} onChange={(e) => toggle([x.key], e.target.checked)} aria-label="Chọn" /></td>
        <td>{LABEL[x.field]}</td>
        <td><Before x={x} /></td><td><After x={x} /></td></tr>,
    );
  }
  return (
    <>
      <table className="diff"><thead><tr><th /><th>Trường</th><th>Cũ</th><th>Mới</th></tr></thead><tbody>{rows}</tbody></table>
      {list.length > limit && <button className="btn" onClick={() => setLimit((l) => l + 1000)}>Hiện thêm ({fmtN(list.length - limit)} còn lại)</button>}
    </>
  );
}

function ByField({ list, on_, toggle }: { list: Item[]; on_: Set<string>; toggle: (k: string[], v: boolean) => void }) {
  const groups = useMemo(() => {
    const g = new Map<string, Item[]>();
    list.forEach((x) => { const id = changeId(x); g.set(id, [...(g.get(id) ?? []), x]); });
    return [...g.values()].sort((a, b) => b.length - a.length);
  }, [list]);
  return (
    <table className="diff"><thead><tr><th /><th>Trường</th><th>Cũ → Mới</th><th>Số file</th></tr></thead>
      <tbody>{groups.slice(0, 500).map((xs) => {
        const ks = xs.map((x) => x.key);
        return (<tr key={ks[0]}><td><input type="checkbox" checked={ks.every((k) => on_.has(k))} onChange={(e) => toggle(ks, e.target.checked)} aria-label="Chọn nhóm" /></td>
          <td>{LABEL[xs[0].field]}</td><td><Before x={xs[0]} /> → <After x={xs[0]} /></td><td className="num">{fmtN(xs.length)}</td></tr>);
      })}</tbody></table>
  );
}

// ------------------------------------------------------------------ history & undo (APP-HIST-*)

let openReviewForUndo: ((runId: number) => void) | null = null;
export function registerUndoOpener(f: (runId: number) => void) { openReviewForUndo = f; }

export async function undoRun(runId: number) {
  const plan = await api.undoPlan(runId);
  const r = store.edit(plan.stage, `Hoàn tác lượt ${runId}`);
  const notes = [
    plan.changedAfter.length ? `bỏ qua ${plan.changedAfter.length} trường đã thay đổi sau lượt này` : "",
    plan.missing.length ? `${plan.missing.length} file không còn tồn tại` : "",
  ].filter(Boolean).join(", ");
  store.notify(`Đã tạo ${r.applied} thay đổi đảo ngược${notes ? ", " + notes : ""}. Xem trước để áp dụng.`);
  if (r.applied) openReviewForUndo?.(runId);
}

export function HistoryDialog({ onClose }: { onClose: () => void }) {
  const [runs, setRuns] = useState<RunInfo[] | null>(null);
  useEffect(() => { api.listRuns().then(setRuns); }, []);
  const fmt = (s: number) => new Date(s * 1000).toLocaleString("vi-VN");
  return (
    <Modal title="Lịch sử" wide onClose={onClose} footer={<><span className="spacer" /><button className="btn primary" onClick={onClose}>Đóng</button></>}>
      {!runs ? <p className="muted">Đang tải…</p> : !runs.length ? <p className="muted">Chưa có lượt áp dụng nào</p> : (
        <table className="diff"><thead><tr><th>#</th><th>Lúc</th><th>Loại</th><th>File</th><th>Tóm tắt</th><th /></tr></thead>
          <tbody>{runs.map((r) => (
            <tr key={r.id}><td className="num">{r.id}</td><td>{fmt(r.started)}</td><td>{r.label ?? "Sửa tag"}</td>
              <td className="num">{fmtN(r.filesOk)}{r.filesConflict + r.filesError ? <span className="muted"> (+{r.filesConflict + r.filesError} lỗi)</span> : null}</td>
              <td>{r.summary}</td>
              <td>{r.status === "undone" ? <span className="pill">đã hoàn tác</span>
                : r.filesOk > 0 && <button className="btn" onClick={() => { onClose(); undoRun(r.id); }}>{r.status === "partially_undone" ? "Hoàn tác phần còn lại…" : "Hoàn tác…"}</button>}</td></tr>
          ))}</tbody></table>
      )}
    </Modal>
  );
}

// ------------------------------------------------------------------ settings (APP-SET)

export function SettingsDialog({ onClose }: { onClose: () => void }) {
  const [mode, setMode] = useState<string>("safe");
  const sources = store.sources;
  const ex = useRef<Record<number, string>>({});
  useEffect(() => { api.getSetting("write_mode").then((v) => setMode(v ?? "safe")); }, []);
  const save = async () => {
    await api.setSetting("write_mode", mode);
    for (const s of sources) if (ex.current[s.id] !== undefined) await api.setExcludes(s.id, ex.current[s.id].split(",").map((x) => x.trim()).filter(Boolean));
    await store.refreshSources();
    store.notify("Đã lưu cài đặt");
    onClose();
  };
  return (
    <Modal title="Cài đặt" onClose={onClose} footer={<><span className="spacer" /><button className="btn" onClick={onClose}>Huỷ</button><button className="btn primary" onClick={save}>Lưu</button></>}>
      <div className="fld"><label>Cách ghi tag</label>
        <label className="checkline"><input type="radio" checked={mode === "safe"} onChange={() => setMode("safe")} />An toàn: ghi vào bản tạm, kiểm tra rồi mới thay file gốc (chậm hơn trên NAS)</label>
        <label className="checkline"><input type="radio" checked={mode === "in_place"} onChange={() => setMode("in_place")} />Ghi thẳng vào file như bộ script cũ (nhanh hơn; vẫn có nhật ký để hoàn tác)</label>
      </div>
      {sources.map((s) => (
        <div className="fld" key={s.id}><label htmlFor={"ex" + s.id}>Thư mục loại trừ của «{s.name}» (phân tách bằng dấu phẩy)</label>
          <input id={"ex" + s.id} defaultValue={s.excludes.join(", ")} onChange={(e) => (ex.current[s.id] = e.target.value)} />
          <button className="btn danger-text" onClick={async () => { await api.removeSource(s.id); await store.init(); onClose(); }}>Gỡ nguồn này khỏi thư viện</button>
        </div>
      ))}
    </Modal>
  );
}

export { K };
