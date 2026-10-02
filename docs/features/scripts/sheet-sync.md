---
status: as-built
app: scripts
---

# Đồng bộ từ bảng tính

## SCR-SYNC-APPLY — Ghi tag đã sửa trên Google Sheet về file

### Là gì?
Công cụ dòng lệnh so sánh trang tính gốc với bản sao đã sửa, rồi ghi các dòng thay đổi vào file nhạc.

### Khi nào dùng?
Sau khi đã sửa tag trên bản sao của trang tính được tạo bởi [SCR-EXPORT-GSHEET](tag-export.md).

### Điều kiện trước
- Trang gốc và trang đã sửa cùng nằm trong bảng tính đích.
- Thứ tự dòng của trang đã sửa giữ nguyên như trang gốc.
- Các file vẫn nằm đúng đường dẫn lúc quét.

### Màn hình / giao diện
Không có.

### Luồng chính
- F1. Khai báo tên trang gốc, tên trang đã sửa, bật ghi tag.
- F2. Công cụ in số dòng mỗi trang và số thay đổi.
- F3. Ghi từng thay đổi, cập nhật bộ đếm tại chỗ.
- F4. Nếu có lỗi, các dòng lỗi được ghi vào trang lỗi.

### Quy tắc nghiệp vụ áp dụng
SYNC-R1, SYNC-R2, SYNC-R3, SYNC-R4, SYNC-R5, SYNC-R7, SYNC-R8, TAGDATA-R5.

### Kiểm tra hợp lệ
Không có kiểm tra số dòng hai trang bằng nhau.

### Trường dữ liệu
File, Directory, Album, Album Artist, Title, Artist, Composer, Genre, Comment.

### Trạng thái
Không áp dụng.

### Ma trận trạng thái – thao tác
Không áp dụng.

### Ma trận quyền – thao tác
Không áp dụng.

### Định dạng số
Số nguyên.

### Trạng thái rỗng & lỗi
- Không có thay đổi: in 0, vẫn tạo trang lỗi rỗng (xem GAP-006).

## SCR-SYNC-MOVE — Chuyển thư mục đã sửa sang khu "Good"

### Là gì?
Bước tuỳ chọn của cùng công cụ. Chuyển các thư mục chứa file có thay đổi sang thư mục con "Good" của thư mục gốc.

### Luồng chính
- F1. Bật chuyển thư mục (có thể tắt ghi tag).
- F2. Công cụ chuyển từng thư mục, in đường dẫn và bộ đếm mỗi 10 thư mục.

### Quy tắc nghiệp vụ áp dụng
SYNC-R6, SYNC-R7, SYNC-R8.

### Trạng thái rỗng & lỗi
Chuyển lỗi: in "Error---------------------------------------" và đường dẫn, tiếp tục.
