// Field metadata and the client-side copy of normalisation/validation (APP-TAG-R2, R4, R6).
export const RIFF_SYNC = "_riffsync";

export const LABEL: Record<string, string> = {
  title: "Title", artist: "Artist", album: "Album", albumartist: "Album Artist", composer: "Composer",
  genre: "Genre", year: "Year", track: "Track", tracktotal: "Track Total", disc: "Disc", disctotal: "Disc Total",
  comment: "Comment", [RIFF_SYNC]: "RIFF INFO",
};

export const TEXT_FIELDS = ["title", "artist", "album", "albumartist", "composer", "genre", "comment"];
export const NUMERIC = new Set(["year", "track", "tracktotal", "disc", "disctotal"]);
export const INSPECTOR_FIELDS = ["title", "artist", "album", "albumartist", "composer", "genre", "year", "track", "tracktotal", "disc", "disctotal", "comment"];
export const BATCH_FIELDS = ["album", "albumartist", "artist", "genre", "year", "composer", "disc", "disctotal", "comment"];

export function normalize(s: string): string {
  return s.normalize("NFC").replace(/\s+/g, " ").trim();
}

export function validate(field: string, v: string): string | null {
  if (v === "") return null;
  if (field === "year" && !(/^\d{4}$/.test(v) && v >= "1000")) return "Năm phải gồm 4 chữ số";
  if (["track", "tracktotal", "disc", "disctotal"].includes(field)) {
    if (!/^\d+$/.test(v) || +v < 1 || +v > 999) return "Giá trị phải là số nguyên từ 1 đến 999";
  }
  return null;
}

/** Remove Vietnamese diacritics for accent-insensitive search. */
export function fold(s: string): string {
  return s.normalize("NFD").replace(/[̀-ͯ]/g, "").replace(/đ/g, "d").replace(/Đ/g, "D").toLowerCase();
}

export const PLACEHOLDERS = /^(unknown artist|unknown album|unknown title|various)$/i;

export function formatDuration(ms: number | null): string {
  if (ms == null) return "";
  const s = Math.round(ms / 1000);
  const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), sec = s % 60;
  return h ? `${h}:${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}` : `${m}:${String(sec).padStart(2, "0")}`;
}

export function formatSize(bytes: number): string {
  if (bytes >= 1e9) return (bytes / 1e9).toLocaleString("vi-VN", { maximumFractionDigits: 1 }) + " GB";
  return (bytes / 1e6).toLocaleString("vi-VN", { maximumFractionDigits: 1 }) + " MB";
}

export const fmtN = (n: number) => n.toLocaleString("vi-VN");
