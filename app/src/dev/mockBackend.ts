// Browser-only stand-in for the Rust commands, used to exercise the UI with `npm run dev`
// outside Tauri. Mirrors the behaviour of db.rs / apply.rs closely enough for UI tests;
// the real logic is covered by the Rust tests.
import { mockIPC } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import fixture from "./fixture.json";
import type { TrackRow } from "../api";

type Staged = { orig: string; value: string };

export function installMockBackend() {
  const tracks = new Map<string, TrackRow>((fixture as TrackRow[]).map((t) => [t.path, structuredClone(t)]));
  const staged = new Map<string, Staged>();
  const runs: any[] = [];
  const journal: { run: number; path: string; field: string; before: string; after: string }[] = [];
  const key = (p: string, f: string) => p + "\u0001" + f;
  const source = { id: 1, path: (fixture as TrackRow[])[0]?.dir.split("/testlib")[0] + "/testlib", name: "testlib (mock)", excludes: ["roon-backup"], status: "ready", lastScan: 0 };
  // one sample with the Decca-style empty "albumartist" key, for the APP-TAG-R13 filter
  const flac = [...tracks.values()].find((t) => t.ext === "flac");
  if (flac) flac.aaMismatch = true;
  (window as any).__mock = { tracks, staged, runs, journal, conflictPath: null as string | null };

  mockIPC(
    async (cmd, args: any) => {
      switch (cmd) {
        case "list_sources": return [source];
        case "load_tracks": return [...tracks.values()];
        case "get_setting": return null;
        case "set_setting": case "set_excludes": case "cancel_scan": case "cancel_apply": return null;
        case "scan_source": return { sourceId: 1, read: 0, added: 0, changed: 0, removed: 0, errors: 0, skipped: 0, cancelled: false, unavailable: false };
        case "stage_set": {
          for (const c of args.changes) {
            const t = tracks.get(c.path);
            if (!t) continue;
            const orig = t.fields[c.field] ?? "";
            if (orig === c.value) staged.delete(key(c.path, c.field));
            else staged.set(key(c.path, c.field), { orig, value: c.value });
          }
          return { applied: args.changes.length, rejected: [] };
        }
        case "stage_list":
          return [...staged].map(([k, s]) => { const [path, field] = k.split("\u0001"); return { path, field, ...s }; });
        case "stage_discard":
          for (const i of args.keys) staged.delete(key(i.path, i.field));
          return null;
        case "apply_run": {
          const id = runs.length + 1;
          const files = [...new Set(args.items.map((i: any) => i.path))] as string[];
          const res: any[] = [];
          let n = 0;
          for (const p of files) {
            await new Promise((r) => setTimeout(r, 120));
            const t = tracks.get(p)!;
            let r: any;
            if ((window as any).__mock.conflictPath === p) {
              r = { path: p, status: "conflict", message: "File đã bị thay đổi bên ngoài app", row: null };
            } else {
              for (const i of args.items.filter((x: any) => x.path === p)) {
                const s = staged.get(key(p, i.field));
                if (!s) continue;
                journal.push({ run: id, path: p, field: i.field, before: t.fields[i.field] ?? "", after: s.value });
                if (i.field === "_riffsync") t.riffMismatch = false;
                else if (i.field === "_aasync") t.aaMismatch = false;
                else { if (s.value) t.fields[i.field] = s.value; else delete t.fields[i.field]; if (i.field === "albumartist") t.aaMismatch = false; }
                staged.delete(key(p, i.field));
              }
              r = { path: p, status: "ok", message: null, row: structuredClone(t) };
            }
            res.push(r);
            await emit("apply-progress", { done: ++n, total: files.length, result: r });
          }
          const ok = res.filter((r) => r.status === "ok").length;
          runs.unshift({ id, kind: args.undoOf ? "undo" : "tags", label: args.undoOf ? `Hoàn tác của lượt ${args.undoOf}` : null, started: Date.now() / 1000, finished: Date.now() / 1000, status: "done", filesOk: ok, filesConflict: res.length - ok, filesError: 0, summary: "", undoOf: args.undoOf ?? null });
          if (args.undoOf && ok) { const o = runs.find((r) => r.id === args.undoOf); if (o) o.status = "undone"; }
          return { runId: id, ok, conflict: res.length - ok, error: 0, cancelled: 0, results: res };
        }
        case "list_runs": return runs;
        case "undo_plan": {
          const stage: any[] = [], changedAfter: any[] = [];
          for (const j of journal.filter((x) => x.run === args.runId && x.field !== "_riffsync")) {
            const cur = tracks.get(j.path)?.fields[j.field] ?? "";
            if (cur === j.after) stage.push({ path: j.path, field: j.field, value: j.before });
            else changedAfter.push({ path: j.path, field: j.field, current: cur, before: j.before });
          }
          return { runId: args.runId, stage, changedAfter, missing: [] };
        }
        default:
          if (cmd.startsWith("plugin:")) return null;
          throw new Error("mock: unknown command " + cmd);
      }
    },
    { shouldMockEvents: true },
  );
}

// ---- test helpers for driving the canvas grid from the console (dev only)
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const canvas = () => document.querySelector("canvas") as HTMLCanvasElement;
const mouse = (type: string, x: number, y: number, buttons: number, extra: MouseEventInit = {}) =>
  canvas().dispatchEvent(new MouseEvent(type, { bubbles: true, cancelable: true, clientX: x, clientY: y, buttons, button: 0, ...extra }));
Object.assign(window as any, {
  __click: async (col: number, row: number, extra: MouseEventInit = {}) => {
    const b = (window as any).__grid.getBounds(col, row);
    const x = b.x + b.width / 2, y = b.y + b.height / 2;
    canvas().focus();
    mouse("mousedown", x, y, 1, extra); mouse("mouseup", x, y, 0, extra); mouse("click", x, y, 0, extra);
    await sleep(150);
  },
  __dragFill: async (col: number, from: number, to: number, alt = false) => {
    const g = (window as any).__grid;
    const b1 = g.getBounds(col, from), b2 = g.getBounds(col, to);
    const hx = b1.x + b1.width - 2, hy = b1.y + b1.height - 2, ty = b2.y + b2.height / 2;
    mouse("mousedown", hx, hy, 1, { altKey: alt });
    for (let y = hy; y <= ty; y += 5) { mouse("mousemove", hx, y, 1, { altKey: alt }); await sleep(15); }
    mouse("mouseup", hx, ty, 0, { altKey: alt });
    await sleep(300);
  },
  __staged: () => [...(window as any).__mock.staged.entries()].map(([k, v]: [string, Staged]) => k.split("/").pop()!.replace("\u0001", " · ") + " = " + v.value),
});
