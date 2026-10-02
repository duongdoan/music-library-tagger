---
status: draft
app: app
---

# Tổ chức file

## APP-FILE-MOVE — Chuyển thư mục đã xử lý

### Là gì?
Chuyển các thư mục album đã chuẩn hoá sang thư mục đích, giữ cấu trúc tương đối. Thay cho bước "chuyển sang Good" của luồng cũ.

### Màn hình / giao diện
- Chọn thư mục trên cây hoặc chọn dòng trong bảng, rồi chuột phải và chọn "Chuyển tới…".
- Hộp thoại: thư mục đích (mặc định theo nguồn), bảng kế hoạch "Thư mục · Đích · Ghi chú", và nút "Chuyển n thư mục".

### Luồng chính
- F1. Chọn thư mục, mở "Chuyển tới…".
- F2. Rà kế hoạch. Các mục "Trùng đích" và "Còn thay đổi chưa áp dụng" bị bỏ chọn.
- F3. Xác nhận. Tiến độ và báo cáo giống lượt áp dụng tag.

### Quy tắc nghiệp vụ áp dụng
APP-FILEOP-R1, R3, R4, R5, R6, R7, R11.

## APP-FILE-CLEANUP — Dọn file album nguyên khối

### Là gì?
Tìm và bỏ các file .flac/.ape nguyên album khi đã có bản tách track.

### Màn hình / giao diện
- Menu Công cụ, chọn "Dọn file thừa…", chọn phạm vi (nguồn hoặc thư mục).
- Bảng kế hoạch: File · Dung lượng · Lý do; tổng dung lượng giải phóng.
- Phần "Không đề xuất" liệt kê các trường hợp bị loại, kèm lý do.

### Luồng chính
- F1. Mở công cụ, chọn phạm vi, nhấn "Phân tích".
- F2. Rà danh sách đề xuất.
- F3. Nhấn "Chuyển n file vào Thùng rác".

### Quy tắc nghiệp vụ áp dụng
APP-FILEOP-R1, R2, R8, R9, R10, R11.

### Định dạng số
Dung lượng "480 MB", tổng "Giải phóng 12,3 GB".
