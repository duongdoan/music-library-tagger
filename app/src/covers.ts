// Cover thumbnails: an in-memory cache in front of the backend's disk cache, and a
// small LIFO queue so scrolling loads what is on screen first without flooding the NAS.
import { api, type Cover, type TrackRow } from "./api";

const MAX_PARALLEL = 6;
const cache = new Map<string, Cover | null>();
const waiting = new Map<string, Array<() => void>>();
const queue: Array<{ key: string; path: string; size: number }> = [];
let running = 0;

const keyOf = (t: TrackRow, size: number) => `${t.path}\u0001${t.mtime}\u0001${size}`;

function pump() {
  while (running < MAX_PARALLEL && queue.length) {
    const job = queue.pop()!; // newest request first: the rows the user is looking at now
    running++;
    api.getCover(job.path, job.size)
      .catch(() => null)
      .then((c) => {
        cache.set(job.key, c);
        const cbs = waiting.get(job.key) ?? [];
        waiting.delete(job.key);
        cbs.forEach((f) => f());
      })
      .finally(() => {
        running--;
        pump();
      });
  }
}

/** Cover if known (null = no image), undefined while loading; `onReady` fires once it arrives. */
export function coverFor(t: TrackRow, size: number, onReady?: () => void): Cover | null | undefined {
  const key = keyOf(t, size);
  if (cache.has(key)) return cache.get(key);
  const cbs = waiting.get(key);
  if (cbs) {
    if (onReady) cbs.push(onReady);
    return undefined;
  }
  waiting.set(key, onReady ? [onReady] : []);
  queue.push({ key, path: t.path, size });
  if (queue.length > 400) {
    // drop the oldest (now off-screen) requests; they are asked again when scrolled back
    for (const j of queue.splice(0, queue.length - 400)) waiting.delete(j.key);
  }
  pump();
  return undefined;
}
