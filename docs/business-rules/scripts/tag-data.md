---
status: as-built
app: scripts
---

# Dữ liệu tag (bộ công cụ script)

## Bối cảnh nghiệp vụ
Thư viện nhạc gồm các file âm thanh nằm trong cây thư mục, thường đặt trên ổ mạng (NAS). Mỗi file mang một bộ thông tin mô tả gọi là **tag** (metadata gắn trong file). Mọi công cụ trong bộ script đều đọc và/hoặc ghi cùng một tập trường tag dưới đây.

## Vai trò tham gia
- **Người quản lý thư viện**: người duy nhất sử dụng công cụ. Không phân quyền.

## Quy trình end-to-end
Xem [SCAN](library-scan.md), [EDIT](tag-edit.md), [SYNC](sheet-sync.md).

## Quy tắc & logic tính toán
- **TAGDATA-R1. Tập trường tag được quản lý.** Các trường: Album, Album Artist, Title, Artist, Composer, Genre, Compilation, Comment, Artwork.
- **TAGDATA-R2. Trường vị trí file.** Mỗi bản ghi kèm theo: File (tên file, không gồm thư mục), Directory (đường dẫn đầy đủ của thư mục chứa file). Một số công cụ có thêm Dir (chỉ tên thư mục cuối cùng chứa file).
- **TAGDATA-R3. Giá trị tag luôn được xử lý dưới dạng chuỗi văn bản.** Trường không có giá trị được biểu diễn thành chuỗi rỗng.
- **TAGDATA-R4. Artwork chỉ được đọc dưới dạng mô tả văn bản** (không xuất hình ảnh). Không có công cụ nào ghi Artwork.
- **TAGDATA-R5. Compilation chỉ đọc, không ghi** ở mọi công cụ ghi tag.
- **TAGDATA-R6. File không đọc được tag bị bỏ qua im lặng.** Gồm file không phải âm thanh (ảnh bìa, .cue, .log, .txt…) và file âm thanh hỏng. Không có thông báo hay thống kê riêng cho các file này.
- **TAGDATA-R7. Mọi file trong cây thư mục đều được thử đọc**, không lọc theo đuôi file trước.

## Ví dụ số minh hoạ
Thư mục có 12 file: 10 file .flac hợp lệ, 1 file .cue, 1 file .jpg.
- Số file được duyệt: 12.
- Số bản ghi tag thu được: 10.
- Số file bị bỏ qua (không thông báo): 2.

## Trạng thái & chuyển trạng thái
Không áp dụng.

## Thông báo lỗi & ràng buộc
| Tình huống | Hành vi |
|---|---|
| File không phải âm thanh | Bỏ qua, không thông báo (TAGDATA-R6) |
| File âm thanh hỏng / định dạng không hỗ trợ | Bỏ qua, không thông báo (TAGDATA-R6) |

## Cấu hình ảnh hưởng hành vi
Không có.

## Tiêu chí chấp nhận
### TAGDATA-V1 — File không phải nhạc bị bỏ qua
- *Kiểm chứng:* TAGDATA-R6, TAGDATA-R7
- *Điều kiện trước:* Thư mục có 2 file .mp3 có tag và 1 file .jpg.
- *Các bước:* Chạy công cụ Quét tag trên thư mục.
- *Kết quả mong đợi:* Kết quả có đúng 2 dòng dữ liệu; không có lỗi hiển thị.
