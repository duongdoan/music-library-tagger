---
status: as-built
app: scripts
---

# Dọn file

## SCR-CLEAN-CUEAPE — Dọn file .flac/.ape nguyên album

### Là gì?
Công cụ dòng lệnh đưa vào Thùng rác các file album nguyên khối (.flac/.ape đi kèm .cue) và mọi file .ape.

### Khi nào dùng?
Sau khi đã tách album thành từng track.

### Điều kiện trước
Thư mục đã khai báo trong công cụ.

### Màn hình / giao diện
Không có. Mỗi file .cue/.ape gặp phải được in đường dẫn.

### Luồng chính
- F1. Chạy công cụ.
- F2. Với từng file .cue: đưa .flac và .ape cùng tên vào Thùng rác.
- F3. Với từng file đuôi "ape": đưa vào Thùng rác.
- F4. In "Finished analyzing, processed n" rồi "DONE".

### Quy tắc nghiệp vụ áp dụng
CLEAN-R1, CLEAN-R2, CLEAN-R3, CLEAN-R4, CLEAN-R5.

### Kiểm tra hợp lệ
Không có.

### Trường dữ liệu
Không áp dụng.

### Trạng thái
Không áp dụng.

### Ma trận trạng thái – thao tác
Không áp dụng.

### Ma trận quyền – thao tác
Không áp dụng.

### Định dạng số
"n/0" (tổng luôn bằng 0).

### Trạng thái rỗng & lỗi
File tương ứng không tồn tại: in lỗi, tiếp tục.
