// Sources, folder tree and review filters (APP-LIB-OPEN, APP-LIB-FILTER).
import { useDeferredValue, useEffect, useMemo, useState } from "react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { store, useStore } from "../store";
import { fmtN } from "../fields";
import { recheckFolder, removeSource, rescan } from "../actions";
import { Confirm } from "./Dialogs";
import type { Source } from "../api";

interface Node { path: string; name: string; count: number; children: Map<string, Node> }

function buildTree(src: Source, dirs: Map<string, number>): Node {
  const root: Node = { path: src.path, name: src.name, count: 0, children: new Map() };
  for (const [dir, n] of dirs) {
    if (dir !== src.path && !dir.startsWith(src.path + "/")) continue;
    root.count += n;
    const rel = dir.slice(src.path.length + 1);
    if (!rel) continue;
    let node = root;
    let acc = src.path;
    for (const part of rel.split("/")) {
      acc += "/" + part;
      let ch = node.children.get(part);
      if (!ch) { ch = { path: acc, name: part, count: 0, children: new Map() }; node.children.set(part, ch); }
      ch.count += n;
      node = ch;
    }
  }
  return root;
}

const STATUS: Record<string, [string, string]> = {
  new: ["Chưa quét", "muted"],
  scanning: ["Đang quét", "warn"],
  ready: ["Sẵn sàng", "ok"],
  cancelled: ["Quét chưa xong", "warn"],
  unavailable: ["Không khả dụng", "danger"],
};

type MenuItem = { label: string; run: () => void; danger?: boolean; disabled?: boolean };

/** "⋯" button that opens a small action menu; closes on outside click or Escape. */
function RowMenu({ items }: { items: MenuItem[] }) {
  const [open, setOpen] = useState(false);
  useEffect(() => {
    if (!open) return;
    const close = () => setOpen(false);
    const esc = (e: KeyboardEvent) => e.key === "Escape" && setOpen(false);
    window.addEventListener("mousedown", close);
    window.addEventListener("keydown", esc);
    return () => { window.removeEventListener("mousedown", close); window.removeEventListener("keydown", esc); };
  }, [open]);
  return (
    <span className={"row-menu" + (open ? " open" : "")} onMouseDown={(e) => e.stopPropagation()} onClick={(e) => e.stopPropagation()}>
      <button className="dots" aria-label="Thao tác" aria-haspopup="menu" aria-expanded={open} onClick={() => setOpen((o) => !o)}>⋯</button>
      {open && (
        <span className="menu" role="menu">
          {items.map((it) => (
            <button key={it.label} role="menuitem" className={it.danger ? "danger" : ""} disabled={it.disabled}
              onClick={() => { setOpen(false); it.run(); }}>{it.label}</button>
          ))}
        </span>
      )}
    </span>
  );
}

function TreeNode({ node, depth, open, toggle, src, onRemove }: {
  node: Node; depth: number; open: Set<string>; toggle: (p: string) => void; src: Source; onRemove: (s: Source) => void;
}) {
  const folder = useStore((s) => s.folder);
  const busy = useStore((s) => !!s.scan || !!s.checking);
  const isOpen = open.has(node.path);
  const isRoot = node.path === src.path;
  const kids = [...node.children.values()].sort((a, b) => a.name.localeCompare(b.name, "vi"));
  const [stLabel, stTone] = STATUS[src.status] ?? [src.status, "muted"];
  const items: MenuItem[] = isRoot
    ? [
        { label: src.status === "unavailable" ? "Thử lại" : src.status === "cancelled" ? "Quét tiếp" : "Quét lại", run: () => rescan(src), disabled: busy },
        { label: "Hiện trong Finder", run: () => revealItemInDir(node.path).catch((e) => store.notify(String(e))) },
        { label: "Gỡ nguồn khỏi thư viện…", run: () => onRemove(src), danger: true, disabled: busy },
      ]
    : [
        { label: "Kiểm tra lại thư mục", run: () => recheckFolder(src, node.path, true), disabled: busy },
        { label: "Hiện trong Finder", run: () => revealItemInDir(node.path).catch((e) => store.notify(String(e))) },
      ];
  return (
    <>
      <div
        role="button"
        tabIndex={0}
        className={"side-item" + (folder === node.path ? " on" : "")}
        style={{ paddingLeft: 8 + depth * 14 }}
        onClick={() => onOpenFolder(node.path)}
        onKeyDown={(e) => e.key === "Enter" && onOpenFolder(node.path)}
        title={isRoot ? `${node.path} · ${stLabel}` : node.path}
      >
        <span className="ind" onClick={(e) => { e.stopPropagation(); toggle(node.path); }}>{kids.length ? (isOpen ? "▾" : "▸") : ""}</span>
        <span className="ellipsis">{node.name}</span>
        {isRoot && <span className={"st-dot " + stTone} aria-label={stLabel} />}
        <span className="cnt">{fmtN(node.count)}</span>
        <RowMenu items={items} />
      </div>
      {isOpen && kids.map((k) => <TreeNode key={k.path} node={k} depth={depth + 1} open={open} toggle={toggle} src={src} onRemove={onRemove} />)}
    </>
  );
}

/** APP-SCAN-R12: opening a folder re-checks it in the background. */
function onOpenFolder(path: string) {
  store.setView({ folder: path });
  const src = store.sources.find((s) => path === s.path || path.startsWith(s.path + "/"));
  if (src && path !== src.path) recheckFolder(src, path);
}

export default function Sidebar({ onAddSource, onHistory }: { onAddSource: () => void; onHistory: () => void }) {
  const sources = useStore((s) => s.sources);
  const version = useStore((s) => s.version);
  const quick = useStore((s) => s.quick);
  const [open, setOpen] = useState<Set<string>>(new Set());
  const [removing, setRemoving] = useState<Source | null>(null);
  const toggle = (p: string) => setOpen((o) => { const n = new Set(o); n.has(p) ? n.delete(p) : n.add(p); return n; });

  const dirs = useMemo(() => {
    const m = new Map<string, number>();
    for (const t of store.tracks) m.set(t.dir, (m.get(t.dir) ?? 0) + 1);
    return m;
  }, [store.tracks]);
  const trees = useMemo(() => sources.map((s) => buildTree(s, dirs)), [sources, dirs]);

  // counts over the whole library: computed after the urgent render so edits stay instant
  const lazyVersion = useDeferredValue(version);
  const counts = useMemo(() => store.counts(), [lazyVersion]);

  return (
    <aside className="side">
      <div>
        <h4>Nguồn <button className="link" onClick={onAddSource} title="Thêm thư mục nhạc">＋</button></h4>
        <button className={"side-item" + (store.folder === null ? " on" : "")} onClick={() => store.setView({ folder: null })}>
          <span className="ind" />Toàn bộ thư viện<span className="cnt">{fmtN(store.tracks.length)}</span>
        </button>
        {trees.map((t, i) => (
          <TreeNode key={t.path} node={t} depth={0} open={open} toggle={toggle} src={sources[i]} onRemove={setRemoving} />
        ))}
        {removing && (
          <Confirm
            text={`Gỡ nguồn «${removing.name}» (${fmtN(store.tracks.filter((t) => t.sourceId === removing.id).length)} file) khỏi thư viện? File nhạc trên ổ đĩa không bị xoá.` +
              (pendingIn(removing) ? ` ${fmtN(pendingIn(removing))} file đang có thay đổi chưa áp dụng sẽ bị bỏ các thay đổi đó.` : "")}
            ok="Gỡ nguồn"
            onClose={() => setRemoving(null)}
            onOk={() => removeSource(removing)}
          />
        )}
      </div>
      <div>
        <h4>Rà soát</h4>
        {store.quickFilters.map((f) => {
          const n = counts.get(f.key) ?? 0;
          return (
            <button key={f.key} title={f.label} className={"side-item" + (quick === f.key ? " on" : "") + (n === 0 && f.key !== "all" ? " zero" : "")} onClick={() => store.setView({ quick: f.key })}>
              <span className="ind" /><span className="ellipsis">{f.label}</span><span className="cnt">{fmtN(n)}</span>
            </button>
          );
        })}
      </div>
      <div>
        <h4>Hoạt động</h4>
        <button className="side-item" onClick={onHistory}><span className="ind" />Lịch sử</button>
      </div>
    </aside>
  );
}

const pendingIn = (src: Source) => store.tracks.filter((t) => t.sourceId === src.id && store.hasPending(t)).length;
