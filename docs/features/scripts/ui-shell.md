---
status: as-built
app: scripts
---

# Khung giao diện (đang dở)

## SCR-UI-DESKTOP — Cửa sổ "Music Tags Reader"

### Là gì?
Cửa sổ ứng dụng desktop, kích thước cố định 800 × 600, có hai tab "Read Tags" và "Write Tags". Chưa nối với chức năng nào.

### Màn hình / giao diện
- Tab "Read Tags":
  - Dòng 1: nhãn "Enter Music Dir", ô nhập, nút "Select" (mở hộp chọn thư mục).
  - Dòng 2: nhãn "Enter Output File", ô nhập, nút "Select" (mở hộp chọn file).
  - Một nhãn trạng thái (chưa dùng).
- Tab "Write Tags": trống.

### Luồng chính
- F1. Nhấn "Select" ở dòng 1, chọn thư mục; đường dẫn được điền vào ô.
- F2. Nhấn "Select" ở dòng 2, chọn file (xem GAP-007).
- Không có nút chạy quét.

### Quy tắc nghiệp vụ áp dụng
Chưa có.

## SCR-UI-WEB — Trang web cục bộ

### Là gì?
Máy chủ web cục bộ, tự mở trình duyệt khi khởi động.

### Luồng chính
- F1. Trang chủ hiển thị "Congratulations, it's a web app!".
- F2. Truy cập trang phân tích sẽ chạy [SCR-EXPORT-GSHEET-LITE](tag-export.md) trên một thư mục cố định. Trang chờ tới khi quét xong rồi hiển thị "Done".

### Quy tắc nghiệp vụ áp dụng
Như SCR-EXPORT-GSHEET-LITE.

## SCR-UI-SIMPLE — Cửa sổ mẫu

### Là gì?
Cửa sổ thử nghiệm với "Some text on Row 1", một ô nhập, nút "Ok" và "Cancel". Không có chức năng nghiệp vụ.
