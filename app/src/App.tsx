import { useEffect, useState } from "react";
import "./App.css";
import { store, useStore } from "./store";
import { addSource, refreshStaleSources, changeCase, CASE_LABEL, cleanText, numberTracks, syncTags, startListeners } from "./actions";
import { api } from "./api";
import { fmtN, LABEL, TEXT_FIELDS } from "./fields";
import Grid from "./components/Grid";
import Sidebar from "./components/Sidebar";
import Inspector from "./components/Inspector";
import { Confirm, HistoryDialog, PatternDialog, registerUndoOpener, ReplaceDialog, ReviewDialog, SettingsDialog } from "./components/Dialogs";

let booted = false; // the restore prompt belongs to app start only (not to dev remounts)

type Dialog = null | { kind: "replace" | "pattern" | "history" | "settings" | "discard" | "case" } | { kind: "review"; undoOf?: number };

export default function App() {
  const [ready, setReady] = useState(false);
  const [dialog, setDialog] = useState<Dialog>(null);
  const [restore, setRestore] = useState(0);
  const sources = useStore((s) => s.sources);
  const query = useStore((s) => s.query);

  useEffect(() => {
    registerUndoOpener((runId) => setDialog({ kind: "review", undoOf: runId }));
    startListeners()
      .then(() => store.init())
      .then(() => {
        // APP-STAGE-R5: offer to restore staged edits left from a previous session
        if (store.staged.size && !booted) setRestore(store.staged.size);
        if (!booted) refreshStaleSources();
        booted = true;
        setReady(true);
      })
      .catch((e) => { store.notify(String(e)); setReady(true); });
  }, []);

  useEffect(() => {
    const f = (e: KeyboardEvent) => {
      const m = e.metaKey || e.ctrlKey;
      const typing = (e.target as HTMLElement)?.closest?.("input, textarea, select, [contenteditable]");
      if (dialog) return;
      if (m && e.key.toLowerCase() === "z" && !typing) { e.preventDefault(); e.shiftKey ? store.redo() : store.undo(); }
      else if (m && e.key.toLowerCase() === "s") { e.preventDefault(); if (store.staged.size) setDialog({ kind: "review" }); }
      else if (m && e.shiftKey && e.key.toLowerCase() === "h") { e.preventDefault(); setDialog({ kind: "replace" }); }
      else if (m && e.key === ",") { e.preventDefault(); setDialog({ kind: "settings" }); }
    };
    window.addEventListener("keydown", f);
    return () => window.removeEventListener("keydown", f);
  }, [dialog]);

  if (!ready) return <div className="loading">Đang mở thư viện…</div>;

  if (!sources.length)
    return (
      <div className="welcome">
        <h1>Music Library Tagger</h1>
        <p>Thêm thư mục nhạc để bắt đầu. App quét tag vào chỉ mục trên máy; với ổ mạng, lần quét đầu có thể mất vài chục phút và có thể dừng rồi chạy tiếp.</p>
        <button className="btn primary big" onClick={addSource}>Thêm thư mục nhạc</button>
        <Toast />
      </div>
    );

  return (
    <div className="app">
      <div className="toolbar">
        <button className="btn" disabled={!store.undoStack.length} onClick={() => store.undo()} title="Hoàn tác (⌘Z)">↶ Hoàn tác</button>
        <button className="btn" disabled={!store.redoStack.length} onClick={() => store.redo()} title="Làm lại (⇧⌘Z)">↷</button>
        <span className="spacer" />
        <button className="btn" onClick={() => setDialog({ kind: "replace" })}>Tìm &amp; thay <kbd>⇧⌘H</kbd></button>
        <button className="btn" onClick={() => setDialog({ kind: "pattern" })}>Tag từ tên file</button>
        <button className="btn" onClick={() => numberTracks()}>Đánh số track</button>
        <button className="btn" onClick={() => setDialog({ kind: "case" })}>Chữ hoa…</button>
        <button className="btn" onClick={cleanText}>Dọn khoảng trắng</button>
        <button className="btn" onClick={syncTags} title="Ghi lại RIFF INFO của WAV theo ID3, và ghi Album Artist vào mọi khoá file đang dùng">Đồng bộ tag lệch</button>
        <input className="search" type="search" placeholder="Tìm title, artist, album…" aria-label="Tìm kiếm" value={query} onChange={(e) => store.setView({ query: e.target.value })} />
        <button className="btn" onClick={() => setDialog({ kind: "settings" })} title="Cài đặt (⌘,)">⚙</button>
      </div>
      <div className="body3">
        <Sidebar onAddSource={addSource} onHistory={() => setDialog({ kind: "history" })} />
        <div className="center"><Grid /></div>
        <Inspector />
      </div>
      <ActionBar onReview={() => setDialog({ kind: "review" })} onDiscard={() => setDialog({ kind: "discard" })} />
      {dialog?.kind === "replace" && <ReplaceDialog onClose={() => setDialog(null)} />}
      {dialog?.kind === "pattern" && <PatternDialog onClose={() => setDialog(null)} />}
      {dialog?.kind === "history" && <HistoryDialog onClose={() => setDialog(null)} />}
      {dialog?.kind === "settings" && <SettingsDialog onClose={() => setDialog(null)} />}
      {dialog?.kind === "review" && <ReviewDialog undoOf={dialog.undoOf} onClose={() => setDialog(null)} />}
      {dialog?.kind === "case" && <CaseDialog onClose={() => setDialog(null)} />}
      {dialog?.kind === "discard" && (
        <Confirm text={`Bỏ ${fmtN(store.staged.size)} thay đổi chưa áp dụng?`} ok="Bỏ tất cả" onClose={() => setDialog(null)}
          onOk={() => store.discard([...store.staged.keys()], "Bỏ tất cả")} />
      )}
      {restore > 0 && (
        <Confirm text={`Khôi phục ${fmtN(restore)} thay đổi chưa áp dụng?`} ok="Khôi phục" cancel="Bỏ các thay đổi đó" onClose={() => setRestore(0)}
          onOk={() => store.notify(`Đã khôi phục ${fmtN(restore)} thay đổi`)}
          onCancel={() => { store.discard([...store.staged.keys()], "Bỏ thay đổi cũ"); store.undoStack = []; }} />
      )}
      <Toast />
    </div>
  );
}

function CaseDialog({ onClose }: { onClose: () => void }) {
  const [field, setField] = useState("title");
  return (
    <div className="modal-bg" onMouseDown={(e) => e.target === e.currentTarget && onClose()}>
      <div className="modal" role="dialog" aria-label="Chuẩn hoá chữ hoa">
        <header><h5>Chuẩn hoá chữ hoa</h5><span className="spacer" /><button className="btn" onClick={onClose}>✕</button></header>
        <div className="mbody">
          <div className="fld"><label htmlFor="cField">Trường (áp lên dòng đang chọn, hoặc toàn bộ dòng đang hiển thị)</label>
            <select id="cField" value={field} onChange={(e) => setField(e.target.value)}>{TEXT_FIELDS.map((f) => <option key={f} value={f}>{LABEL[f]}</option>)}</select></div>
          <div className="row wrap">{Object.entries(CASE_LABEL).map(([k, l]) => <button key={k} className="btn" onClick={() => { changeCase(k, field); onClose(); }}>{l}</button>)}</div>
        </div>
      </div>
    </div>
  );
}

function ActionBar({ onReview, onDiscard }: { onReview: () => void; onDiscard: () => void }) {
  const n = useStore((s) => s.staged.size);
  const files = useStore((s) => s.pendingFiles());
  const scan = useStore((s) => s.scan);
  const checking = useStore((s) => s.checking);
  const fill = useStore((s) => (s.lastFill && s.lastOp() === s.lastFill.op ? s.lastFill : null));
  return (
    <div className="actionbar">
      {scan ? (
        <div className="progress">
          <span>
            {scan.phase === "list"
              ? `Đang liệt kê «${scan.name}» — ${fmtN(scan.done)} file`
              : `Đang quét «${scan.name}» — ${fmtN(scan.done)} / ${fmtN(scan.total)} file · ${Math.round(scan.perSec)} file/giây` +
                (scan.perSec > 0 && scan.total > scan.done ? ` · còn khoảng ${eta((scan.total - scan.done) / scan.perSec)}` : "")}
          </span>
          {scan.phase === "read" && <div className="bar"><i style={{ width: `${(scan.done / Math.max(1, scan.total)) * 100}%` }} /></div>}
          <button className="btn" onClick={() => api.cancelScan()}>Huỷ</button>
        </div>
      ) : (
        <span className="ab-count">{n ? <><b>{fmtN(n)} thay đổi</b> trên {fmtN(files)} file</> : "Không có thay đổi chờ"}
          {checking && <span className="muted small"> · đang kiểm tra «{checking}»…</span>}</span>
      )}
      {fill && (
        <span className="fillopt">Tuỳ chọn điền:
          <button className={"chip" + (fill.mode === "copy" ? " on" : "")} onClick={() => fill.redo("copy")}>Sao chép ô</button>
          <button className={"chip" + (fill.mode === "series" ? " on" : "")} onClick={() => fill.redo("series")}>Điền chuỗi</button>
        </span>
      )}
      <span className="spacer" />
      <button className="btn" disabled={!n} onClick={onDiscard}>Bỏ tất cả</button>
      <button className="btn primary" disabled={!n} onClick={onReview}>Xem trước &amp; áp dụng ({fmtN(n)}) <kbd>⌘S</kbd></button>
    </div>
  );
}

function eta(sec: number) {
  if (sec < 60) return `${Math.ceil(sec)} giây`;
  const m = Math.floor(sec / 60);
  return m < 60 ? `${m} phút ${Math.round(sec % 60)} giây` : `${Math.floor(m / 60)} giờ ${m % 60} phút`;
}

function Toast() {
  const t = useStore((s) => s.toast);
  const [shown, setShown] = useState<number | null>(null);
  useEffect(() => {
    if (!t) return;
    setShown(t.id);
    const h = setTimeout(() => setShown(null), 4000);
    return () => clearTimeout(h);
  }, [t?.id]);
  return t && shown === t.id ? <div className="toast" role="status">{t.text}</div> : null;
}
