---
status: draft
app: app
---

# Xem trước & áp dụng

## APP-APPLY-REVIEW — Xem trước thay đổi

### Là gì?
Màn hình rà soát mọi thay đổi chờ trước khi ghi vào file.

### Khi nào dùng?
Mỗi lần muốn ghi. Đây là con đường duy nhất để ghi (APP-WRITE-R1).

### Điều kiện trước
Có ít nhất 1 thay đổi chờ.

### Màn hình / giao diện
- Mở bằng nút "Xem trước & áp dụng (n)" ở thanh hành động, hoặc Cmd+S.
- Danh sách thay đổi nhóm theo thư mục, rồi theo file. Mỗi dòng: Trường · Giá trị cũ (gạch ngang) · Giá trị mới (tô sáng). Mỗi cấp nhóm có ô chọn.
- Tab "Theo trường": gom thay đổi theo trường và giá trị, ví dụ "Album: Bolero 4 → Bolero Tuyển Chọn IV (12 file)". Bỏ chọn một nhóm sẽ bỏ cả nhóm.
- Tóm tắt: "n thay đổi trên m file · k thư mục".
- Nút "Áp dụng n thay đổi" và "Đóng".

### Luồng chính
- F1. Mở Xem trước.
- F2. Bỏ chọn các mục không muốn ghi. Mục bỏ chọn vẫn là thay đổi chờ, chỉ không ghi trong lượt này.
- F3. Nhấn "Áp dụng" để chuyển sang APP-APPLY-RUN.

### Quy tắc nghiệp vụ áp dụng
APP-WRITE-R1, APP-WRITE-R2.

### Kiểm tra hợp lệ
| Khi nào kiểm tra | Điều kiện hợp lệ | Thông báo khi không hợp lệ |
|---|---|---|
| Nhấn Áp dụng | Có ít nhất 1 mục được chọn | (nút bị vô hiệu) |

### Trường dữ liệu
File, Thư mục, Trường, Giá trị cũ, Giá trị mới.

### Trạng thái
Không áp dụng.

### Ma trận trạng thái – thao tác
Không áp dụng.

### Ma trận quyền – thao tác
Không áp dụng.

### Định dạng số
Phân cách nghìn.

### Trạng thái rỗng & lỗi
Không có thay đổi chờ: nút ở thanh hành động bị vô hiệu.

## APP-APPLY-RUN — Ghi thay đổi

### Màn hình / giao diện
Tiến độ trong cửa sổ Xem trước: "Đang ghi n / tổng file", thanh tiến độ, và nút "Huỷ". Từng file được đánh dấu xong hoặc lỗi theo thời gian thực.

### Luồng chính
- F1. App lần lượt sao lưu, kiểm tra xung đột, ghi, xác minh từng file.
- F2. Người dùng có thể huỷ.
- F3. Kết thúc thì chuyển sang APP-APPLY-REPORT.

### Quy tắc nghiệp vụ áp dụng
APP-WRITE-R2 đến APP-WRITE-R10.

## APP-APPLY-REPORT — Báo cáo lượt áp dụng

### Màn hình / giao diện
- Tóm tắt (APP-WRITE-R11).
- Danh sách file không thành công kèm lý do. Chuột phải để chọn "Thử lại", "Lấy giá trị trong file" hoặc "Giữ thay đổi của tôi" (với Xung đột).
- Nút "Hoàn tác lượt này", "Xem trong lịch sử", "Đóng".

### Quy tắc nghiệp vụ áp dụng
APP-WRITE-R11, APP-STAGE (trạng thái Lỗi ghi, Xung đột), APP-UNDO-R1.

### Trạng thái rỗng & lỗi
Thành công hoàn toàn: "Đã ghi n thay đổi vào m file." và nút "Hoàn tác".
