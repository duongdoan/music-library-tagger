// Right panel: one track, or the batch form for several (APP-LIB-INSPECT, APP-EDIT-BATCH).
import { useEffect, useState } from "react";
import { store, useStore } from "../store";
import { BATCH_FIELDS, formatDuration, formatSize, INSPECTOR_FIELDS, LABEL, normalize, validate } from "../fields";
import type { Cover, TrackRow } from "../api";
import { coverFor } from "../covers";

export default function Inspector() {
  const selected = useStore((s) => s.selected);
  useStore((s) => s.version);
  const rows = selected.map((p) => store.byPath.get(p)).filter(Boolean) as TrackRow[];
  if (!rows.length) return <aside className="insp"><p className="hint">Chọn một track để xem chi tiết.<br /><br />Chọn nhiều track để sửa hàng loạt.</p></aside>;
  if (rows.length === 1) return <aside className="insp"><Single t={rows[0]} /></aside>;
  return <aside className="insp"><Batch rows={rows} key={selected.join("|").slice(0, 200) + selected.length} /></aside>;
}

function Single({ t }: { t: TrackRow }) {
  const st = store.status.get(t.path);
  if (t.error)
    return (<><h5>{t.file}</h5><div className="sub">{t.dir}</div><div className="errbox">{t.error}. File này không sửa được cho tới khi quét lại thành công.</div></>);
  if (t.readonly)
    return (<><h5>{t.file}</h5><div className="sub">{t.dir}</div><div className="infobox">Không hỗ trợ sửa tag định dạng {t.ext.toUpperCase()}.</div></>);
  return (
    <>
      <CoverView t={t} />
      {st && (
        <div className={st.status === "conflict" ? "confbox" : "errbox"}>
          <b>{st.status === "conflict" ? "Xung đột" : "Lỗi ghi"}</b> {st.message}
          <span className="row">
            <button className="chip" onClick={() => { store.status.delete(t.path); store.notify("Giữ thay đổi của bạn, sẽ ghi ở lượt sau"); }}>Giữ thay đổi của tôi</button>
            <button className="chip" onClick={() => {
              const keys = [...store.staged.keys()].filter((k) => k.startsWith(t.path + "\u0001"));
              store.status.delete(t.path);
              store.discard(keys, "Lấy giá trị trong file");
            }}>Lấy giá trị trong file</button>
          </span>
        </div>
      )}
      {t.riffMismatch && <div className="infobox">RIFF INFO của file WAV này lệch với ID3. Dùng "Đồng bộ tag lệch" để sửa.</div>}
      {t.aaMismatch && <div className="infobox">Album Artist được lưu ở nhiều khoá với giá trị khác nhau (ví dụ một khoá trống). Trình phát như Roon có thể đọc khoá trống. Dùng "Đồng bộ tag lệch" hoặc sửa Album Artist để ghi cùng một giá trị vào mọi khoá.</div>}
      <div><h5>{t.file}</h5><div className="sub">{t.dir}</div></div>
      <div className="fields">
        {INSPECTOR_FIELDS.map((k) => <FieldInput key={t.path + k} t={t} field={k} />)}
      </div>
      <dl className="ro-info">
        <dt>Định dạng</dt><dd>{t.format}</dd>
        <dt>Thời lượng</dt><dd>{formatDuration(t.durationMs)}</dd>
        <dt>Dung lượng</dt><dd>{formatSize(t.size)}</dd>
      </dl>
    </>
  );
}

function FieldInput({ t, field }: { t: TrackRow; field: string }) {
  const value = store.val(t, field);
  const [v, setV] = useState(value);
  const [err, setErr] = useState<string | null>(null);
  useEffect(() => setV(value), [value]);
  const pending = store.isPending(t, field);
  const commit = () => {
    const n = normalize(v);
    const e = validate(field, n) ?? cross(t, field, n);
    setErr(e);
    if (!e && n !== value) store.edit([{ path: t.path, field, value: n }], "Sửa " + LABEL[field]);
  };
  return (
    <div className={"fld" + (["track", "tracktotal", "disc", "disctotal"].includes(field) ? " half" : "")}>
      <label htmlFor={"f_" + field}>{LABEL[field]}</label>
      <input id={"f_" + field} value={v} className={pending ? "pend" : ""} onChange={(e) => setV(e.target.value)} onBlur={commit}
        onKeyDown={(e) => { if (e.key === "Enter") (e.target as HTMLInputElement).blur(); if (e.key === "Escape") setV(value); }} />
      {pending && <span className="orig">{t.fields[field] || "(trống)"}</span>}
      {err && <span className="err">{err}</span>}
    </div>
  );
}

function cross(t: TrackRow, field: string, v: string) {
  const tr = field === "track" ? v : store.val(t, "track");
  const tt = field === "tracktotal" ? v : store.val(t, "tracktotal");
  return tr && tt && +tr > +tt ? "Track không được lớn hơn Track Total" : null;
}

type Action = "keep" | "set" | "clear";

function Batch({ rows }: { rows: TrackRow[] }) {
  const ok = rows.filter((t) => store.editable(t));
  const skipped = rows.length - ok.length;
  const init = () => Object.fromEntries(BATCH_FIELDS.map((k) => {
    const vals = new Set(ok.map((t) => store.val(t, k)));
    return [k, { act: "keep" as Action, value: vals.size === 1 ? [...vals][0] : "" }];
  }));
  const [form, setForm] = useState<Record<string, { act: Action; value: string }>>(init);
  const [errs, setErrs] = useState<Record<string, string>>({});
  const set = (k: string, p: Partial<{ act: Action; value: string }>) => setForm((f) => ({ ...f, [k]: { ...f[k], ...p } }));

  const apply = () => {
    const e: Record<string, string> = {};
    const plan: [string, string][] = [];
    for (const k of BATCH_FIELDS) {
      const { act, value } = form[k];
      if (act === "keep") continue;
      const v = act === "clear" ? "" : normalize(value);
      const er = validate(k, v);
      if (er) e[k] = er;
      plan.push([k, v]);
    }
    setErrs(e);
    if (Object.keys(e).length || !plan.length) return;
    const r = store.edit(ok.flatMap((t) => plan.map(([field, value]) => ({ path: t.path, field, value }))), "Sửa nhiều file");
    store.notify(r.applied ? `Đã tạo ${r.applied} thay đổi chờ trên ${ok.length} file` : "Không có gì thay đổi");
    setForm(init());
  };

  return (
    <>
      <div>
        <h5>Sửa {ok.length.toLocaleString("vi-VN")} file</h5>
        <div className="hint-sm">Chỉ trường đặt hành động khác "Giữ nguyên" mới tạo thay đổi.{skipped ? ` Đã bỏ qua ${skipped} file (lỗi đọc hoặc không khả dụng).` : ""}</div>
      </div>
      {BATCH_FIELDS.map((k) => {
        const counts = new Map<string, number>();
        ok.forEach((t) => { const v = store.val(t, k); counts.set(v, (counts.get(v) ?? 0) + 1); });
        const same = counts.size === 1;
        return (
          <div className="fld" key={k}>
            <label htmlFor={"b_" + k}>
              {LABEL[k]}
              <select className="act" value={form[k].act} onChange={(e) => set(k, { act: e.target.value as Action })} aria-label={"Hành động cho " + LABEL[k]}>
                <option value="keep">Giữ nguyên</option><option value="set">Đặt giá trị</option><option value="clear">Xoá</option>
              </select>
            </label>
            <input id={"b_" + k} value={form[k].value} placeholder={same ? "" : "(nhiều giá trị)"}
              onChange={(e) => set(k, { value: e.target.value, act: "set" })} />
            {!same && (
              <div className="chips">
                {[...counts].sort((a, b) => b[1] - a[1]).slice(0, 4).map(([v, n]) => (
                  <button className="chip" key={v} onClick={() => set(k, { value: v, act: "set" })}>{v || "(trống)"} ({n})</button>
                ))}
              </div>
            )}
            {errs[k] && <span className="err">{errs[k]}</span>}
          </div>
        );
      })}
      <div className="sticky-foot"><button className="btn primary" onClick={apply}>Áp vào {ok.length.toLocaleString("vi-VN")} file</button></div>
    </>
  );
}

/** Large cover with its size and where it comes from (APP-LIB-INSPECT). */
function CoverView({ t }: { t: TrackRow }) {
  const [c, setC] = useState<Cover | null | undefined>(() => coverFor(t, 600));
  useEffect(() => {
    let live = true;
    const now = coverFor(t, 600, () => live && setC(coverFor(t, 600)));
    setC(now);
    return () => { live = false; };
  }, [t.path, t.mtime]);
  if (c === undefined) return <div className="art loading" aria-busy="true">Đang tải ảnh bìa…</div>;
  if (c === null) return <div className="art none">Không có ảnh bìa</div>;
  const kb = c.bytes >= 1e6 ? (c.bytes / 1e6).toLocaleString("vi-VN", { maximumFractionDigits: 1 }) + " MB" : Math.round(c.bytes / 1024) + " KB";
  return (
    <figure className="cover">
      <img src={c.url} alt={`Ảnh bìa ${t.fields.album ?? ""}`} />
      <figcaption>
        {c.width}×{c.height} · {kb} · {c.source === "embedded" ? "nhúng trong file" : `từ ${c.file ?? "file ảnh"} trong thư mục`}
        {c.width < 500 && <span className="warn"> · ảnh nhỏ</span>}
      </figcaption>
    </figure>
  );
}
