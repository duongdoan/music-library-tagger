---
status: draft
app: app
---

# Áp dụng thay đổi vào file

## Bối cảnh nghiệp vụ
Ghi tag là thao tác thay đổi dữ liệu thật trên thư viện. Lỗi ở bước này có thể làm sai lệch hàng nghìn file. App phải cho xem trước, ghi an toàn từng file, phát hiện file bị sửa từ bên ngoài, và luôn hoàn tác được.

## Vai trò tham gia
- **Người quản lý thư viện.**

## Quy trình end-to-end
1. Mở Xem trước thay đổi ([APP-APPLY-REVIEW](../../features/app/apply.md)).
2. Bỏ chọn các thay đổi không muốn ghi.
3. Áp dụng ([APP-APPLY-RUN](../../features/app/apply.md)).
4. Xem báo cáo ([APP-APPLY-REPORT](../../features/app/apply.md)). Nếu cần, hoàn tác ([APP-HIST-UNDO](../../features/app/history.md)).

## Quy tắc & logic tính toán
- **APP-WRITE-R1. Bắt buộc xem trước.** Không có cách ghi nào bỏ qua màn hình Xem trước thay đổi.
- **APP-WRITE-R2. Đơn vị ghi là file.** Mỗi file được ghi một lần với mọi thay đổi đã chọn của file đó.
- **APP-WRITE-R3. Sao lưu trước khi ghi.** Trước khi ghi một file, app lưu giá trị hiện tại của các trường sắp đổi (kể cả Artwork) vào **nhật ký** của lượt áp dụng. Nếu không lưu được nhật ký thì không ghi file.
- **APP-WRITE-R4. Phát hiện xung đột.** Ngay trước khi ghi, app so thời điểm sửa và dung lượng file với chỉ mục.
  - Nếu khác: file không được ghi và chuyển sang trạng thái "Xung đột" (APP-STAGE).
- **APP-WRITE-R5. Ghi trọn vẹn hoặc không ghi.** Một file sau lượt ghi phải ở một trong hai tình trạng: có đủ mọi thay đổi, hoặc giữ nguyên như trước. Không chấp nhận file bị ghi dở.
- **APP-WRITE-R6. Xác minh sau ghi.** Sau khi ghi, app đọc lại tag của file.
  - Khớp giá trị mới: cập nhật chỉ mục, thay đổi chuyển "Đồng bộ".
  - Không khớp: "Lỗi ghi" với lý do "Giá trị sau khi ghi không khớp".
- **APP-WRITE-R7. Lỗi một file không dừng lượt ghi.** File lỗi giữ nguyên thay đổi chờ (trạng thái "Lỗi ghi") để thử lại.
- **APP-WRITE-R8. Tiến độ và huỷ.**
  - Hiển thị "Đang ghi n / tổng file".
  - Huỷ sẽ dừng sau file đang ghi. File chưa ghi giữ nguyên thay đổi chờ.
- **APP-WRITE-R9. Không đổi thời điểm sửa (tuỳ chọn).** Khi bật "Giữ nguyên ngày sửa file", app đặt lại thời điểm sửa của file về giá trị trước khi ghi. Mặc định tắt.
- **APP-WRITE-R10. Ghi tuần tự trên ổ mạng.** Với nguồn trên ổ mạng, ghi từng file một. Với ổ cục bộ, được ghi song song tối đa 4 file.
- **APP-WRITE-R11. Báo cáo.** Kết thúc lượt ghi, hiển thị: số file thành công, lỗi, xung đột, đã huỷ, cùng danh sách chi tiết các file không thành công.

## Ví dụ số minh hoạ
Có 1.200 thay đổi chờ trên 400 file. Người dùng bỏ chọn 1 album 12 file (36 thay đổi).
- Lượt ghi: 388 file, 1.164 thay đổi.
- Trong lúc chờ, một file đã bị app khác sửa, và 2 file đang chỉ đọc.
- Kết quả: thành công 385, xung đột 1, lỗi 2. Báo cáo: "Đã ghi 385 / 388 file. 1 xung đột, 2 lỗi."
- Còn lại 36 + (3 file × số thay đổi của chúng) thay đổi chờ.

## Trạng thái & chuyển trạng thái
Trạng thái lượt áp dụng: Đang chạy, sau đó Hoàn tất / Hoàn tất có lỗi / Đã huỷ. Trạng thái từng trường: xem [APP-STAGE](staging.md).

## Thông báo lỗi & ràng buộc
| Tình huống | Thông báo |
|---|---|
| File chỉ đọc / không có quyền | "Không có quyền ghi file" |
| File bị thay đổi sau lần quét | "File đã bị thay đổi bên ngoài app" |
| Ổ mạng mất kết nối giữa chừng | "Mất kết nối tới «nguồn»". Các file còn lại chuyển về chờ áp dụng. |
| Không lưu được nhật ký | "Không thể sao lưu tag gốc, đã dừng ghi" (dừng cả lượt) |
| Xác minh không khớp | "Giá trị sau khi ghi không khớp" |

## Cấu hình ảnh hưởng hành vi
- "Giữ nguyên ngày sửa file" (APP-WRITE-R9).

## Tiêu chí chấp nhận
### APP-WRITE-V1 — Xung đột được phát hiện
- *Kiểm chứng:* APP-WRITE-R4
- *Điều kiện trước:* File A có thay đổi chờ Title. Dùng app khác sửa Artist của A.
- *Các bước:* Áp dụng.
- *Kết quả mong đợi:* A không bị ghi; trạng thái "Xung đột"; báo cáo "1 xung đột".

### APP-WRITE-V2 — Lỗi một file không dừng lượt ghi
- *Kiểm chứng:* APP-WRITE-R7, APP-WRITE-R11
- *Điều kiện trước:* 10 file có thay đổi chờ, 1 file đặt chỉ đọc.
- *Các bước:* Áp dụng.
- *Kết quả mong đợi:* 9 thành công, 1 lỗi "Không có quyền ghi file"; thay đổi của file lỗi vẫn ở trạng thái chờ.

### APP-WRITE-V3 — Huỷ giữa chừng
- *Kiểm chứng:* APP-WRITE-R8
- *Điều kiện trước:* 500 file có thay đổi chờ.
- *Các bước:* Áp dụng, huỷ khi tiến độ khoảng 100.
- *Kết quả mong đợi:* Khoảng 100 file đã ghi và có trong nhật ký; số còn lại vẫn chờ áp dụng.
