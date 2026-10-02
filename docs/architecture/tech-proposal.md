# Đề xuất kỹ thuật — Music Library Tagger v2

## Yêu cầu phi chức năng
- **Đẹp**: giao diện hiện đại, tuỳ biến tự do, có chế độ sáng/tối.
- **Nhẹ**: bộ cài nhỏ, ít RAM, mở nhanh.
- **Mạnh**:
  - bảng mượt với hơn 50.000 dòng;
  - quét song song, quét tăng dần;
  - ghi có nhật ký, ghi atomic;
  - ổn định trên NAS.
- **Đủ định dạng**: FLAC, MP3, M4A/ALAC, OGG/Opus, AIFF, WAV, APE, WavPack, DSF (WMA là ưu tiên thấp).

## So sánh

Dung lượng và RAM là mức điển hình, chỉ để so sánh tương đối.

| Phương án | Đẹp | Nhẹ | Mạnh | Thư viện tag | Ghi chú |
|---|---|---|---|---|---|
| **Tauri 2 + Rust + React** (khuyến nghị) | 5 | 5 | 5 | lofty | ~10 MB, RAM ~120 MB. Bảng kiểu Excel có sẵn. Cần spike DSF/WMA |
| PySide6/PyQt6 + mutagen | 3 | 3 | 4 | mutagen (đủ định dạng) | ~100 MB. Tận dụng Python. Đẹp cần nhiều công QSS/QML |
| Electron + Node | 5 | 1 | 3 | music-metadata (chỉ đọc) | ~150 MB, RAM 300 MB+. Ghi tag yếu |
| Flutter desktop | 4 | 3 | 3 | TagLib qua FFI | Ít thư viện tag |
| SwiftUI | 5 | 5 | 4 | TagLib qua bridge | Chỉ chạy trên macOS |
| Rust thuần (Slint/egui) | 3 | 5 | 5 | lofty | Khó làm đẹp, ít component bảng |

## Định dạng trong thư viện thật (nas1, đo ngày 01/10/2026)
73.111 file, trong đó 54.797 file âm thanh. Liệt kê toàn bộ qua SMB mất 212 giây.

| Định dạng | Số file | Tỉ lệ | `lofty` (Rust) | `mutagen` (Python) |
|---|---:|---:|---|---|
| FLAC | 39.302 | 71,7% | Có | Có |
| WAV | 12.562 | 22,9% | Có (ID3 + RIFF INFO) | Có |
| DSF | 1.033 | 1,9% | **Không**, tự viết | Có |
| M4A | 938 | 1,7% | Có | Có |
| AIFF | 828 | 1,5% | Có | Có |
| APE | 104 | 0,2% | Có | Có |
| WMA | 30 | 0,05% | **Không** | Có |
| DFF, MP3 | 0 / dưới 10 | — | — | — |

Riêng `lofty` đã phủ 53.734 / 54.797 file (98,1%). Thêm module DSF tự viết thì phủ 54.767 / 54.797 file (99,95%). 30 file WMA chỉ liệt kê, không sửa tag (APP-TAG-R11).

Lấy mẫu 30 file WAV thì 29 file có cả ID3 lẫn RIFF INFO, nên app phải ghi đồng bộ cả hai vùng (APP-TAG-R10).

**Kết luận:** giữ phương án Tauri + Rust.

## Khuyến nghị: Tauri 2 + lõi Rust + React (Glide Data Grid)

| Lớp | Công nghệ |
|---|---|
| Giao diện | React + TypeScript. **Glide Data Grid**: vẽ bằng canvas, có sẵn chọn vùng ô, nút kéo điền, sao chép/dán dạng tab (tương thích Excel). Điền chuỗi số và bỏ qua dòng ẩn/lỗi (APP-STAGE-R16, R17) viết thêm. Phương án thay thế: RevoGrid |
| Tìm & thay | `fancy-regex`: crate `regex` không có lookahead/lookbehind (APP-STAGE-R9). Chuyển cú pháp `$<ten>` sang `${ten}` trước khi thay |
| Cầu nối | Tauri commands. Events/channels stream tiến độ và kết quả quét theo lô 500 dòng |
| Tag | `lofty` |
| Quét | `walkdir` + `rayon`. Song song có giới hạn; trên NAS giới hạn 4–8 luồng |
| Lưu trữ | SQLite (`rusqlite`): chỉ mục, thay đổi chờ (APP-STAGE-R5), lịch sử và nhật ký (APP-UNDO) |
| Chuẩn hoá | `unicode-normalization` (NFC, APP-TAG-R4) |
| Thùng rác | crate `trash`; dự phòng `.mlt-trash` trên NAS |
| Đóng gói | .dmg universal, .msi; Tauri updater |

### Ghi an toàn (APP-WRITE)
1. Ghi nhật ký giá trị gốc vào SQLite, trong một transaction.
2. So mtime và dung lượng với chỉ mục. Lệch thì báo xung đột.
3. File ≤ 500 MB: sao chép sang bản tạm cùng thư mục, ghi tag vào bản tạm, rồi đổi tên đè file gốc (atomic). File lớn hơn (DSF): ghi tại chỗ, dựa vào nhật ký để khôi phục.
4. Đọc lại tag để xác minh, rồi cập nhật chỉ mục.

### Mô hình dữ liệu (SQLite)
- `sources`
- `tracks`: path là khoá chính; mtime, size, các cột tag, artwork_hash.
- `staged_changes`: path, field, orig, new.
- `apply_runs`
- `journal`: run_id, path, field, before, after.
- `file_ops`

### Kết quả spike (01/10/2026)
Xem [spike-report.md](spike-report.md). Các quyết định rút ra:
- **Ghi tag:** dùng API riêng cho từng định dạng, không dùng API tổng quát của `lofty` (API này làm mất tag lạ):
  - FLAC, M4A: `lofty` `FlacFile` / `Mp4File`;
  - vùng ID3 của WAV, AIFF, DSF: crate `id3`;
  - RIFF INFO và DSF: hai module tự viết, đã chạy đúng trên file thật.
- **Đọc khi quét:** bộ đọc nhanh tối giản cho FLAC. Quét lần đầu khoảng 45 phút qua SMB, nên quét phải tiếp tục được sau khi thoát app và ưu tiên thư mục đang mở (APP-SCAN-R11, R12).
- **Glide Data Grid:** `onFillPattern` + `preventDefault` đủ để tự viết logic kéo điền. Không cần RevoGrid.
- **Còn mở:** ghi qua bản tạm rồi đổi tên trên SMB, và tốc độ ghi qua SMB. Nếu chép cả file qua mạng quá chậm, chuyển sang ghi tại chỗ có nhật ký cho mọi file.

### Rủi ro và spike (2–3 ngày, trước khi chốt)
- Glide Data Grid: kiểm tra có chặn được thao tác kéo điền để tự xử lý điền chuỗi số không. Nếu không, dùng RevoGrid hoặc tự viết nút kéo trên bảng ảo hoá.
- DSF (1.033 file): tự viết module nhỏ. Header DSF chứa offset tới khối ID3v2 ở cuối file; dùng crate `id3` để đọc/ghi khối đó, rồi cập nhật offset và kích thước file. Ước lượng 2–4 ngày kèm test trên bản sao file thật.
- Liệt kê tuần tự qua SMB đạt 345 file/giây (73.111 file mất 212 giây). Cần đo `jwalk` duyệt song song, và tốc độ đọc tag FLAC/WAV, để biết thời gian quét lần đầu và quét tăng dần.
- Kiểm tra `lofty` với cả hai kiểu tên vùng ID3 trong WAV ("ID3 " và "id3 ") và ghi đồng bộ RIFF INFO.
- Kiểm tra rename atomic trên SMB share.
- Nếu spike không đạt: chuyển sang PySide6 + mutagen, giữ nguyên spec và kiến trúc lớp (core, storage, UI).

## Lộ trình
| Giai đoạn | Phạm vi | Ước lượng |
|---|---|---|
| 0. Spike | lofty trên mẫu thật, tốc độ NAS, ghi atomic | 2–3 ngày |
| 1. Đọc | Nguồn, quét tăng dần, chỉ mục, bảng, bộ lọc, inspector | 1,5 tuần |
| 2. Sửa | Thay đổi chờ, sửa ô/nhiều file, tìm & thay, tag từ tên file, đánh số, undo/redo | 2 tuần |
| 3. Ghi | Xem trước, lượt ghi, xung đột, xác minh, báo cáo, lịch sử/hoàn tác | 1,5 tuần |
| 4. Tổ chức file | Chuyển thư mục, dọn .cue/.ape, cài đặt, đóng gói | 1 tuần |

Prototype tương tác và sơ đồ luồng: [prototype/music-tagger.html](../prototype/music-tagger.html).
