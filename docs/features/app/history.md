---
status: draft
app: app
---

# Lịch sử

## APP-HIST-LIST — Danh sách lượt áp dụng

### Là gì?
Danh sách các lượt ghi tag và thao tác file, mới nhất ở trên.

### Màn hình / giao diện
- Mục "Lịch sử" trên sidebar.
- Mỗi dòng: thời điểm, loại (Sửa tag / Chuyển thư mục / Dọn file / Hoàn tác), số file, tóm tắt, trạng thái.
- Chọn một dòng để xem chi tiết: danh sách file, trường, giá trị trước và sau.

### Quy tắc nghiệp vụ áp dụng
APP-UNDO-R1, APP-UNDO-R6, APP-FILEOP-R11.

### Trạng thái rỗng & lỗi
"Chưa có lượt áp dụng nào".

## APP-HIST-UNDO — Hoàn tác

### Luồng chính
- F1. Chọn một mục lịch sử, nhấn "Hoàn tác…", hoặc chọn một số file/trường rồi nhấn "Hoàn tác mục đã chọn".
- F2. App tạo thay đổi đảo ngược và mở Xem trước (APP-APPLY-REVIEW). Trường bị sửa sau lượt đó có nhãn "Đã thay đổi sau lượt này" và không được chọn sẵn.
- F3. Áp dụng như bình thường.

### Quy tắc nghiệp vụ áp dụng
APP-UNDO-R2, R3, R4, R5; APP-FILEOP-R11.
