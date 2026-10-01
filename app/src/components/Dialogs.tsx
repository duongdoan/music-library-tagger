// Modal dialogs: find & replace, tag from filename, review & apply, history, settings, confirm.
import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { api, on, type ApplyReport, type FileResult, type RunInfo, type StageInput, type TrackRow } from "../api";
import { store, K } from "../store";
import { AA_SYNC, fmtN, LABEL, normalize, RIFF_SYNC, TEXT_FIELDS } from "../fields";

const PSEUDO_TEXT: Record<string, string> = { [RIFF_SYNC]: "Ghi lại RIFF INFO theo ID3", [AA_SYNC]: "Ghi Album Artist vào mọi khoá đang dùng" };

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
  const [tab, setTab] = useState<"file" | "field">("file");
  const [running, setRunning] = useState<{ done: number; total: number } | null>(null);
  const [results, setResults] = useState<FileResult[]>([]);
  const [report, setReport] = useState<ApplyReport | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const toggle = (keys: string[], v: boolean) => setOn((o) => { const n = new Set(o); keys.forEach((k) => (v ? n.add(k) : n.delete(k))); return n; });
  const files = new Set(list.filter((x) => on_.has(x.key)).map((x) => x.path)).size;
  const dirs = new Set(list.filter((x) => on_.has(x.key)).map((x) => x.t.dir)).size;

  const run = async () => {
    const items = list.filter((x) => on_.has(x.key)).map((x) => ({ path: x.path, field: x.field }));
    setRunning({ done: 0, total: new Set(items.map((i) => i.path)).size });
    const un = await on.applyProgress((p) => {
      setRunning({ done: p.done, total: p.total });
      setResults((r) => [...r, p.result]);
    });
    try {
      const rep = await api.applyRun(items, undoOf);
      setReport(rep);
      for (const r of rep.results) {
        if (r.status === "ok") store.status.delete(r.path);
        else if (r.status !== "cancelled") store.status.set(r.path, { status: r.status, message: r.message });
      }
      store.mergeRows(rep.results.filter((r) => r.row && r.status === "ok").map((r) => r.row!), []);
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
        <div className="progress grow"><span>Đang ghi {fmtN(running.done)} / {fmtN(running.total)} file</span><div className="bar"><i style={{ width: `${(running.done / Math.max(1, running.total)) * 100}%` }} /></div><button className="btn" onClick={() => api.cancelApply()}>Huỷ</button></div>
      ) : (
        <><span><b>{fmtN(on_.size)}</b> thay đổi trên <b>{fmtN(files)}</b> file · {fmtN(dirs)} thư mục</span><span className="spacer" /><button className="btn" onClick={onClose}>Đóng</button><button className="btn primary" disabled={!on_.size} onClick={run}>Áp dụng {fmtN(on_.size)} thay đổi</button></>
      )}>
      {running ? (
        <table className="diff"><tbody>{results.slice(-300).map((r) => (
          <tr key={r.path}><td className="mono small">{r.path.split("/").pop()}</td>
            <td>{r.status === "ok" ? <span className="pill ok">Đã ghi · đã xác minh</span> : r.status === "conflict" ? <span className="pill warn">Xung đột</span> : r.status === "error" ? <span className="pill danger">{r.message}</span> : <span className="pill">Đã huỷ</span>}</td></tr>
        ))}</tbody></table>
      ) : (
        <>
          <div className="tabs"><button className={tab === "file" ? "on" : ""} onClick={() => setTab("file")}>Theo file</button><button className={tab === "field" ? "on" : ""} onClick={() => setTab("field")}>Theo trường</button></div>
          {tab === "file" ? <ByFile list={list} on_={on_} toggle={toggle} /> : <ByField list={list} on_={on_} toggle={toggle} />}
        </>
      )}
    </Modal>
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
        {PSEUDO_TEXT[x.field] ? <td colSpan={2}>{PSEUDO_TEXT[x.field]}{x.field === AA_SYNC ? ` («${x.t.fields.albumartist ?? ""}»)` : ""}</td> : <><td className="old">{show(x.orig)}</td><td><span className="new">{show(x.value)}</span></td></>}</tr>,
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
    list.forEach((x) => { const id = x.field + "\u0000" + x.orig + "\u0000" + x.value; g.set(id, [...(g.get(id) ?? []), x]); });
    return [...g.values()].sort((a, b) => b.length - a.length);
  }, [list]);
  return (
    <table className="diff"><thead><tr><th /><th>Trường</th><th>Cũ → Mới</th><th>Số file</th></tr></thead>
      <tbody>{groups.slice(0, 500).map((xs) => {
        const ks = xs.map((x) => x.key);
        return (<tr key={ks[0]}><td><input type="checkbox" checked={ks.every((k) => on_.has(k))} onChange={(e) => toggle(ks, e.target.checked)} aria-label="Chọn nhóm" /></td>
          <td>{LABEL[xs[0].field]}</td><td>{PSEUDO_TEXT[xs[0].field] ? PSEUDO_TEXT[xs[0].field] : <><span className="old">{show(xs[0].orig)}</span> → <span className="new">{show(xs[0].value)}</span></>}</td><td className="num">{fmtN(xs.length)}</td></tr>);
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
