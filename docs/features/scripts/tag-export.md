---
status: as-built
app: scripts
---

# Quét & xuất tag

## SCR-EXPORT-GSHEET — Quét thư mục ra Google Sheet (bản đầy đủ)

### Là gì?
Công cụ chạy dòng lệnh. Đọc tag của mọi file trong một thư mục (đệ quy) và ghi thành bảng trên một trang tính Google mới.

### Khi nào dùng?
Bước đầu của luồng sửa tag qua bảng tính ([SYNC](../../business-rules/scripts/sheet-sync.md)).

### Điều kiện trước
- Có quyền truy cập bảng tính Google đích bằng tài khoản dịch vụ đã khai báo.
- Thư mục gốc đã được khai báo trong công cụ.
- Chưa có trang tính trùng tên đường dẫn thư mục.

### Màn hình / giao diện
Không có giao diện. Chỉ có dòng tiến độ trên cửa sổ dòng lệnh.

### Luồng chính
- F1. Người dùng sửa thư mục gốc trong công cụ, rồi chạy.
- F2. Công cụ in "Analyzing", tạo trang tính mới, ghi dòng tiêu đề.
- F3. Công cụ duyệt từng file, cập nhật tiến độ "n/tổng" tại chỗ.
- F4. Kết thúc: in "Finished analyzing, processed n" rồi "DONE".

### Quy tắc nghiệp vụ áp dụng
TAGDATA-R1, TAGDATA-R2, TAGDATA-R3, TAGDATA-R4, TAGDATA-R6, TAGDATA-R7, SCAN-R1, SCAN-R2, SCAN-R3, SCAN-R4, SCAN-R5, SCAN-R6, SCAN-R7, SCAN-R8.

### Kiểm tra hợp lệ
Không có.

### Trường dữ liệu
Dir, File, Album, Album Artist, Title, Artist, Composer, Genre, Compilation, Comment, Artwork, Directory.

### Trạng thái
Không áp dụng.

### Ma trận trạng thái – thao tác
Không áp dụng.

### Ma trận quyền – thao tác
Không áp dụng (một người dùng).

### Định dạng số
Tiến độ dạng "n/tổng", số nguyên.

### Trạng thái rỗng & lỗi
- Thư mục rỗng: trang tính chỉ có dòng tiêu đề; "processed 0".
- Trang tính trùng tên: công cụ dừng với lỗi kỹ thuật.

## SCR-EXPORT-GSHEET-LITE — Quét ra Google Sheet (bản rút gọn, dùng cho giao diện web)

### Là gì?
Biến thể của SCR-EXPORT-GSHEET, không có cột "Dir". Được gọi từ trang web cục bộ ([SCR-UI-WEB](ui-shell.md)).

### Khác biệt so với SCR-EXPORT-GSHEET
- 11 cột tiêu đề (SCAN-R3, bản rút gọn).
- Tiến độ in mỗi 100 file (SCAN-R6).
- Tham số được truyền từ nơi gọi, không khai báo trong công cụ.

### Quy tắc nghiệp vụ áp dụng
Như SCR-EXPORT-GSHEET.

## SCR-EXPORT-EXCEL — Quét thư mục ra file Excel

### Là gì?
Biến thể ghi ra file Excel cục bộ thay vì Google Sheet.

### Khác biệt so với SCR-EXPORT-GSHEET
- Đích là một file Excel khai báo trong công cụ. Tên trang tính theo SCAN-R2 (bản Excel).
- 11 cột, không có "Dir".
- Bắt đầu in "Progress" thay vì "Analyzing".
- Hành vi lưu file hiện chưa xác nhận được (xem GAP-002, GAP-003).

### Quy tắc nghiệp vụ áp dụng
TAGDATA-R1 đến TAGDATA-R7, SCAN-R1 đến SCAN-R7.
