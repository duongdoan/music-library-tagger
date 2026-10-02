# Music Library Tagger — Tài liệu đặc tả

## Hai ứng dụng trong tài liệu
| Mã | Ứng dụng | Trạng thái |
|---|---|---|
| `scripts` | Bộ công cụ dòng lệnh hiện tại. Sửa tag qua Google Sheet / Excel. | as-built (hiện trạng) |
| `app` | Ứng dụng desktop độc lập. Mọi thao tác diễn ra trong app, không dùng bảng tính bên ngoài. | draft (đề xuất) |

## Ứng dụng desktop (đề xuất)
**Features**
- [Thư viện](features/app/library.md): APP-LIB-OPEN, APP-LIB-SCAN, APP-LIB-BROWSE, APP-LIB-FILTER, APP-LIB-INSPECT
- [Sửa tag](features/app/edit.md): APP-EDIT-INLINE, APP-EDIT-FILL, APP-EDIT-BATCH, APP-EDIT-REPLACE, APP-EDIT-PATTERN, APP-EDIT-NUMBER, APP-EDIT-CASE, APP-EDIT-RIFFSYNC, APP-EDIT-ARTWORK, APP-EDIT-REVERT
- [Xem trước & áp dụng](features/app/apply.md): APP-APPLY-REVIEW, APP-APPLY-RUN, APP-APPLY-REPORT
- [Lịch sử](features/app/history.md): APP-HIST-LIST, APP-HIST-UNDO
- [Tổ chức file](features/app/file-ops.md): APP-FILE-MOVE, APP-FILE-CLEANUP
- [Cài đặt](features/app/settings.md): APP-SET

**Business rules**
- [Mô hình dữ liệu tag](business-rules/app/tag-model.md): APP-TAG
- [Quét & chỉ mục thư viện](business-rules/app/library-scan.md): APP-SCAN
- [Thay đổi chờ & sửa hàng loạt](business-rules/app/staging.md): APP-STAGE
- [Áp dụng thay đổi vào file](business-rules/app/write-apply.md): APP-WRITE
- [Lịch sử & hoàn tác](business-rules/app/history-undo.md): APP-UNDO
- [Chuyển thư mục & dọn file](business-rules/app/file-ops.md): APP-FILEOP

## Bộ công cụ script (hiện trạng)
**Features**
- [Quét & xuất tag](features/scripts/tag-export.md)
- [Sửa tag hàng loạt](features/scripts/batch-edit.md)
- [Đồng bộ từ bảng tính](features/scripts/sheet-sync.md)
- [Dọn file](features/scripts/file-cleanup.md)
- [Khung giao diện](features/scripts/ui-shell.md)
- [Tiện ích phụ](features/scripts/utilities.md)

**Business rules:** [TAGDATA](business-rules/scripts/tag-data.md) · [SCAN](business-rules/scripts/library-scan.md) · [EDIT](business-rules/scripts/tag-edit.md) · [SYNC](business-rules/scripts/sheet-sync.md) · [CLEAN](business-rules/scripts/file-cleanup.md)

## Đối chiếu tính năng cũ và mới
| Hiện trạng (scripts) | Ứng dụng mới | Thay đổi chính |
|---|---|---|
| Quét ra Google Sheet / Excel | APP-LIB-SCAN + chỉ mục cục bộ | Quét tăng dần, không cần bảng tính |
| Sửa trên Google Sheet / Excel | APP-LIB-BROWSE, APP-EDIT-INLINE, APP-EDIT-FILL, APP-EDIT-BATCH, APP-LIB-FILTER | Sửa ngay trong bảng của app: vùng ô, kéo điền, Cmd+C/V qua lại với Excel; có bộ lọc rà soát |
| Replace bằng regex trong Excel | APP-EDIT-REPLACE | Nhóm bắt $1, $<ten>; xem trước; mẫu đã lưu |
| Đồng bộ sheet về file | APP-APPLY-* | Ghép theo đường dẫn, xem trước, phát hiện xung đột, xác minh |
| Gán tag (Album/Album Artist/Genre) | APP-EDIT-BATCH | Mọi trường, có "Giữ nguyên" |
| Tìm & thay | APP-EDIT-REPLACE | 5 kiểu khớp, xem trước trực tiếp |
| Chuyển sang "Good" | APP-FILE-MOVE | Chặn khi còn thay đổi chờ, không ghi đè |
| Dọn .cue/.ape | APP-FILE-CLEANUP | Chỉ dọn khi bản tách đủ; không bao giờ dọn .ape đơn lẻ |
| (không có) | APP-HIST-*, APP-EDIT-PATTERN, APP-EDIT-NUMBER, APP-EDIT-CASE, APP-EDIT-ARTWORK | Tính năng mới |
| Crawl blog, Fshare | (ngoài phạm vi) | Tách khỏi app |

## Khác
- [Gap Log](qa/gap-log.md)
- [Đề xuất kỹ thuật](architecture/tech-proposal.md)
