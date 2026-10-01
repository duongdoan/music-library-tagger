---
status: as-built
app: scripts
---

# Quét thư viện & xuất bảng tag (bộ công cụ script)

## Bối cảnh nghiệp vụ
Để rà soát và sửa tag cho hàng nghìn file, người quản lý cần có toàn bộ tag ở dạng bảng. Bộ script xuất bảng này ra Google Sheet, hoặc ra file Excel. Người dùng sửa trực tiếp trên bảng đó.

## Vai trò tham gia
- **Người quản lý thư viện.**

## Quy trình end-to-end
1. Khai báo thư mục gốc và nơi xuất (bảng tính Google hoặc file Excel) trong công cụ.
2. Chạy quét: [SCR-EXPORT-GSHEET](../../features/scripts/tag-export.md), [SCR-EXPORT-GSHEET-LITE](../../features/scripts/tag-export.md), [SCR-EXPORT-EXCEL](../../features/scripts/tag-export.md).
3. Mở bảng tính để rà soát và sửa (chuyển sang [SYNC](sheet-sync.md)).

## Quy tắc & logic tính toán
- **SCAN-R1. Quét đệ quy.** Duyệt toàn bộ thư mục gốc và mọi thư mục con, theo thứ tự hệ thống tệp trả về (không sắp xếp).
- **SCAN-R2. Một trang tính mới cho mỗi lần quét.** Lần quét tạo một trang tính mới trong bảng tính đích. Tên trang tính:
  - Google Sheet: đường dẫn đầy đủ của thư mục gốc.
  - Excel: đường dẫn đầy đủ, thay mỗi ký tự "/" bằng "--".
- **SCAN-R3. Dòng tiêu đề.** Dòng 1 là tiêu đề cột, theo thứ tự:
  - Bản đầy đủ: "Dir", "File", "Album", "Album Artist", "Title", "Artist", "Composer", "Genre", "Compilation", "Comment", "Artwork", "Directory".
  - Bản rút gọn và bản Excel: như trên nhưng không có "Dir".
- **SCAN-R4. Ghi theo lô 100 dòng.** Dữ liệu được ghi từ dòng 2, mỗi lần 100 dòng. Phần dư cuối cùng (dưới 100 dòng) được ghi khi kết thúc.
- **SCAN-R5. Đếm trước (tuỳ chọn).** Khi bật "đếm trước", công cụ đếm tổng số file trước khi quét để hiển thị tiến độ "đã xử lý / tổng". Khi tắt, tổng hiển thị là 0.
- **SCAN-R6. Tiến độ.**
  - Bản đầy đủ và Excel: cập nhật tại chỗ sau mỗi file.
  - Bản rút gọn: in "Processed n/tổng" sau mỗi 100 file.
- **SCAN-R7. Số file đã xử lý tính mọi file đã duyệt**, kể cả file bị bỏ qua theo TAGDATA-R6.
- **SCAN-R8. Bảng tính Google đích cố định.** Mọi công cụ dùng chung một bảng tính Google và một tài khoản dịch vụ khai báo sẵn.

## Ví dụ số minh hoạ
Thư mục có 250 file, trong đó 230 file đọc được tag.
- Số dòng dữ liệu: 230. Ghi thành 3 lô: 100 (dòng 2–101), 100 (dòng 102–201), 30 (dòng 202–231).
- Thông báo cuối: "Finished analyzing, processed 250".

## Trạng thái & chuyển trạng thái
Không áp dụng.

## Thông báo lỗi & ràng buộc
| Tình huống | Hành vi / thông báo |
|---|---|
| Đã tồn tại trang tính cùng tên (quét lại cùng thư mục) | Công cụ dừng với lỗi từ dịch vụ bảng tính; không có thông báo thân thiện |
| Bắt đầu quét | "Analyzing" (bản đầy đủ) / "Progress" (bản Excel) |
| Kết thúc | "Finished analyzing, processed n", sau đó "DONE" |

## Cấu hình ảnh hưởng hành vi
- Thư mục gốc, bảng tính đích, đếm trước: khai báo cố định trong công cụ, phải sửa trước khi chạy.

## Tiêu chí chấp nhận
### SCAN-V1 — Xuất đủ dòng theo lô
- *Kiểm chứng:* SCAN-R3, SCAN-R4
- *Điều kiện trước:* Thư mục có 150 file nhạc hợp lệ.
- *Các bước:* Chạy quét bản đầy đủ.
- *Kết quả mong đợi:* Trang tính mới có dòng tiêu đề 12 cột và 150 dòng dữ liệu (dòng 2–151).
