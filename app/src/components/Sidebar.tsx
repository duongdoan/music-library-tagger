// Sources, folder tree and review filters (APP-LIB-OPEN, APP-LIB-FILTER).
import { useMemo, useState } from "react";
import { store, useStore } from "../store";
import { fmtN } from "../fields";
import { rescan } from "../actions";
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

function TreeNode({ node, depth, open, toggle }: { node: Node; depth: number; open: Set<string>; toggle: (p: string) => void }) {
  const folder = useStore((s) => s.folder);
  const isOpen = open.has(node.path);
  const kids = [...node.children.values()].sort((a, b) => a.name.localeCompare(b.name, "vi"));
  return (
    <>
      <button
        className={"side-item" + (folder === node.path ? " on" : "")}
        style={{ paddingLeft: 8 + depth * 14 }}
        onClick={() => onOpenFolder(node.path)}
        title={node.path}
      >
        <span className="ind" onClick={(e) => { e.stopPropagation(); toggle(node.path); }}>{kids.length ? (isOpen ? "▾" : "▸") : ""}</span>
        <span className="ellipsis">{node.name}</span>
        <span className="cnt">{fmtN(node.count)}</span>
      </button>
      {isOpen && kids.map((k) => <TreeNode key={k.path} node={k} depth={depth + 1} open={open} toggle={toggle} />)}
    </>
  );
}

/** APP-SCAN-R12: opening a folder re-checks it in the background. */
function onOpenFolder(path: string) {
  store.setView({ folder: path });
  const src = store.sources.find((s) => path === s.path || path.startsWith(s.path + "/"));
  if (src && !store.scan && path !== src.path) {
    rescan(src, path, true);
  }
}

export default function Sidebar({ onAddSource, onHistory }: { onAddSource: () => void; onHistory: () => void }) {
  const sources = useStore((s) => s.sources);
  const version = useStore((s) => s.version);
  const quick = useStore((s) => s.quick);
  const [open, setOpen] = useState<Set<string>>(new Set());
  const toggle = (p: string) => setOpen((o) => { const n = new Set(o); n.has(p) ? n.delete(p) : n.add(p); return n; });

  const dirs = useMemo(() => {
    const m = new Map<string, number>();
    for (const t of store.tracks) m.set(t.dir, (m.get(t.dir) ?? 0) + 1);
    return m;
  }, [store.tracks]);
  const trees = useMemo(() => sources.map((s) => buildTree(s, dirs)), [sources, dirs]);

  const counts = useMemo(() => {
    const ctx = store.filterCtx();
    return new Map(store.quickFilters.map((f) => [f.key, store.tracks.reduce((n, t) => n + (f.test(t, ctx) ? 1 : 0), 0)]));
  }, [version]);

  return (
    <aside className="side">
      <div>
        <h4>Nguồn <button className="link" onClick={onAddSource} title="Thêm thư mục nhạc">＋</button></h4>
        <button className={"side-item" + (store.folder === null ? " on" : "")} onClick={() => store.setView({ folder: null })}>
          <span className="ind" />Toàn bộ thư viện<span className="cnt">{fmtN(store.tracks.length)}</span>
        </button>
        {trees.map((t, i) => (
          <div key={t.path}>
            <TreeNode node={t} depth={0} open={open} toggle={toggle} />
            <SourceStatus src={sources[i]} />
          </div>
        ))}
      </div>
      <div>
        <h4>Rà soát</h4>
        {store.quickFilters.map((f) => {
          const n = counts.get(f.key) ?? 0;
          return (
            <button key={f.key} className={"side-item" + (quick === f.key ? " on" : "") + (n === 0 && f.key !== "all" ? " zero" : "")} onClick={() => store.setView({ quick: f.key })}>
              <span className="ind" />{f.label}<span className="cnt">{fmtN(n)}</span>
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

function SourceStatus({ src }: { src: Source }) {
  const label: Record<string, string> = { new: "Chưa quét", scanning: "Đang quét", ready: "Sẵn sàng", cancelled: "Quét chưa xong", unavailable: "Không khả dụng" };
  return (
    <div className={"src-status s-" + src.status}>
      {label[src.status] ?? src.status}
      {!store.scan && (
        <button className="link" onClick={() => rescan(src)}>
          {src.status === "unavailable" ? "Thử lại" : src.status === "cancelled" ? "Quét tiếp" : "Quét lại"}
        </button>
      )}
    </div>
  );
}
