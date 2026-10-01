---
status: as-built
app: scripts
---

# Sửa tag hàng loạt

## SCR-EDIT-SET — Gán Album / Album Artist / Genre cho cả thư mục

### Là gì?
Công cụ dòng lệnh ghi cùng một bộ giá trị Album, Album Artist, Genre cho mọi file trong thư mục (đệ quy).

### Khi nào dùng?
Khi chuẩn hoá một tuyển tập. Ví dụ: Album "Bolero Tuyển Chọn IV", Album Artist "Various Artists", Genre "Nhạc vàng".

### Điều kiện trước
Thư mục và ba giá trị đã được khai báo trong công cụ.

### Màn hình / giao diện
Không có. Dòng lệnh in đường dẫn từng file đã ghi và "Processed n/tổng".

### Luồng chính
- F1. Công cụ in "Preparing" và "Total number of files: n".
- F2. Ghi ba trường cho từng file, in đường dẫn file.
- F3. In "Finished setting tags" rồi "DONE".

### Quy tắc nghiệp vụ áp dụng
EDIT-R1, EDIT-R2, EDIT-R7, EDIT-R8, TAGDATA-R6.

### Kiểm tra hợp lệ
Không có. Giá trị rỗng vẫn được ghi.

### Trường dữ liệu
Album, Album Artist, Genre.

### Trạng thái
Không áp dụng.

### Ma trận trạng thái – thao tác
Không áp dụng.

### Ma trận quyền – thao tác
Không áp dụng.

### Định dạng số
"n/tổng".

### Trạng thái rỗng & lỗi
File không ghi được bị bỏ qua, không thông báo.

## SCR-EDIT-REPLACE — Tìm & thay trong một trường tag

### Là gì?
Công cụ dòng lệnh tìm các file có giá trị một trường bắt đầu bằng chuỗi cho trước, rồi thay chuỗi đó bằng chuỗi mới.

### Khi nào dùng?
Khi xoá tiền tố rác hoặc sửa lỗi đặt tên lặp lại. Ví dụ: xoá "16 / " ở đầu Album.

### Điều kiện trước
Đã khai báo thư mục, trường, chuỗi tìm, chuỗi thay, bật/tắt thực hiện thay.

### Màn hình / giao diện
Không có. Mỗi file khớp được in dạng "cũ  ====>  mới".

### Luồng chính
- F1. Chạy với tắt thực hiện thay để xem trước.
- F2. Kiểm tra danh sách cặp cũ/mới.
- F3. Bật thực hiện thay, chạy lại.
- F4. Đọc "Finished checking, found n" và "Finished replacing, replaced m".

### Quy tắc nghiệp vụ áp dụng
EDIT-R3, EDIT-R4, EDIT-R5, EDIT-R6, EDIT-R7, EDIT-R8.

### Kiểm tra hợp lệ
Không có.

### Trường dữ liệu
Một trường bất kỳ trong TAGDATA-R1.

### Trạng thái
Không áp dụng.

### Ma trận trạng thái – thao tác
Không áp dụng.

### Ma trận quyền – thao tác
Không áp dụng.

### Định dạng số
Tiến độ in mỗi 100 file: "Processed n/tổng - Replaced m".

### Trạng thái rỗng & lỗi
Không có file khớp: "found 0", "replaced 0".
