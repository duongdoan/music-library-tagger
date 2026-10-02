---
status: as-built
app: scripts
---

# Đồng bộ chỉnh sửa từ bảng tính về file (bộ công cụ script)

## Bối cảnh nghiệp vụ
Đây là luồng sửa tag chính hiện nay. Người dùng sửa tag trên Google Sheet, sau đó công cụ so sánh bản gốc với bản đã sửa và ghi thay đổi vào file nhạc. Thư mục đã xử lý có thể được chuyển sang khu vực "Good".

## Vai trò tham gia
- **Người quản lý thư viện.**

## Quy trình end-to-end
1. Quét thư mục ra trang tính gốc ([SCR-EXPORT-GSHEET](../../features/scripts/tag-export.md)). Tên trang = đường dẫn thư mục, ví dụ "/Volumes/nas2/Download/Staging".
2. Người dùng nhân bản trang tính gốc, đổi tên bản sao (ví dụ "/Volumes/nas2/Download/Staging-Fixed") và sửa tag trên bản sao.
3. Chạy đồng bộ ([SCR-SYNC-APPLY](../../features/scripts/sheet-sync.md)).
4. Tuỳ chọn: chuyển thư mục đã sửa ([SCR-SYNC-MOVE](../../features/scripts/sheet-sync.md)).

## Quy tắc & logic tính toán
- **SYNC-R1. Ghép dòng theo vị trí.** Dòng thứ i của trang gốc được so với dòng thứ i của trang đã sửa.
- **SYNC-R2. Điều kiện là "thay đổi".** Dòng i là thay đổi khi **cả hai** điều kiện đúng:
  - "File" ở hai trang giống nhau;
  - ít nhất một trong 7 trường khác nhau: Album, Album Artist, Title, Artist, Composer, Genre, Comment.
- **SYNC-R3. Dòng có "File" khác nhau bị bỏ qua im lặng.**
- **SYNC-R4. Ghi toàn bộ 7 trường.** Với mỗi thay đổi, file tại đường dẫn "Directory" + "File" được ghi lại cả 7 trường ở SYNC-R2 theo giá trị trang đã sửa, kể cả trường không đổi. Compilation không được ghi (TAGDATA-R5).
- **SYNC-R5. Nhật ký lỗi.** Mỗi lần chạy tạo một trang tính "<tên trang gốc> Errors <dấu thời gian>". File ghi lỗi được thêm một dòng gồm: File, Album, Album Artist, Title, Artist, Composer, Genre, Directory, nội dung lỗi.
- **SYNC-R6. Chuyển thư mục.** Khi bật, mỗi "Directory" phân biệt trong danh sách thay đổi được chuyển tới đường dẫn mới. Đường dẫn mới là "Directory" sau khi thay chuỗi thư mục gốc bằng "thư mục gốc/Good".
- **SYNC-R7. Hai bước ghi tag và chuyển thư mục bật/tắt độc lập.**
- **SYNC-R8. Không sao lưu, không hoàn tác.**

## Ví dụ số minh hoạ
Thư mục gốc "/Volumes/nas2/Download/Staging". Trang gốc có 3 dòng. Trên bản sao, người dùng sửa Album dòng 2 và Genre dòng 3. Dòng 3 nằm trong thư mục ".../Staging/Album B".
- Số thay đổi: 2.
- File dòng 2 và dòng 3 được ghi lại đủ 7 trường.
- Khi bật chuyển thư mục: ".../Staging/Album B" chuyển thành ".../Staging/Good/Album B".

Trường hợp người dùng sắp xếp lại bản sao theo Album: các dòng lệch vị trí, nên "File" khác nhau và các dòng đó bị bỏ qua (SYNC-R3).

## Trạng thái & chuyển trạng thái
Không áp dụng.

## Thông báo lỗi & ràng buộc
| Tình huống | Hành vi / thông báo |
|---|---|
| Bắt đầu | In "Preparing", số dòng gốc, số dòng đã sửa, số thay đổi, sau đó "Replaced: " |
| Ghi tag lỗi | Ghi vào trang lỗi (SYNC-R5), tiếp tục |
| Chuyển thư mục lỗi | In "Error---------------------------------------" và đường dẫn, tiếp tục |
| Tiến độ chuyển | In số thư mục đã chuyển, mỗi 10 thư mục |

## Cấu hình ảnh hưởng hành vi
- Tên trang gốc, tên trang đã sửa, bật/tắt ghi tag, bật/tắt chuyển thư mục: khai báo cố định trong công cụ.

## Tiêu chí chấp nhận
### SYNC-V1 — Chỉ dòng có thay đổi được ghi
- *Kiểm chứng:* SYNC-R1, SYNC-R2, SYNC-R4
- *Điều kiện trước:* Như ví dụ số minh hoạ, chưa sắp xếp lại bản sao.
- *Các bước:* Chạy đồng bộ, bật ghi tag, tắt chuyển thư mục.
- *Kết quả mong đợi:* In số thay đổi 2; tag của file dòng 2 và 3 khớp bản sao; file dòng 1 không bị ghi.

### SYNC-V2 — Chuyển thư mục sang khu "Good"
- *Kiểm chứng:* SYNC-R6
- *Điều kiện trước:* Như SYNC-V1.
- *Các bước:* Chạy đồng bộ, tắt ghi tag, bật chuyển thư mục.
- *Kết quả mong đợi:* Thư mục "Album B" nằm dưới ".../Staging/Good/".
