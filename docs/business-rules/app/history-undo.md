---
status: draft
app: app
---

# Lịch sử & hoàn tác lượt áp dụng

## Bối cảnh nghiệp vụ
Sau khi ghi, người dùng có thể phát hiện mình sửa nhầm, đôi khi vài ngày sau. Mỗi lượt áp dụng được lưu thành một mục lịch sử để hoàn tác.

## Vai trò tham gia
- **Người quản lý thư viện.**

## Quy trình end-to-end
[APP-HIST-LIST](../../features/app/history.md), sau đó [APP-HIST-UNDO](../../features/app/history.md).

## Quy tắc & logic tính toán
- **APP-UNDO-R1. Một lượt áp dụng là một mục lịch sử.** Mỗi mục gồm: thời điểm, số file, số thay đổi, tóm tắt (ví dụ "Album: 120 file, Genre: 40 file") và nhật ký giá trị gốc (APP-WRITE-R3).
- **APP-UNDO-R2. Hoàn tác chỉ áp lên file đã ghi thành công** trong lượt đó.
- **APP-UNDO-R3. Hoàn tác cũng là một lượt áp dụng.** Nó đi qua Xem trước (APP-WRITE-R1), phát hiện xung đột và xác minh như mọi lượt ghi. Hoàn tác tạo mục lịch sử mới, có nhãn "Hoàn tác của «mục gốc»".
- **APP-UNDO-R4. Xung đột khi hoàn tác.** Trường nào có giá trị hiện tại khác giá trị lượt gốc đã ghi (do bị sửa sau đó) sẽ được đánh dấu "Đã thay đổi sau lượt này". Trường đó mặc định bỏ chọn trong Xem trước.
- **APP-UNDO-R5. Hoàn tác một phần.** Có thể hoàn tác cả lượt, một số file, hoặc một số trường.
- **APP-UNDO-R6. Lưu giữ.** Giữ mục lịch sử tối đa 90 ngày hoặc 200 mục, tuỳ điều kiện nào đến trước. Mục cũ nhất bị xoá trước.

## Ví dụ số minh hoạ
Ngày 1: lượt L1 đổi Album của 12 file "Bolero 4" thành "Bolero Tuyển Chọn IV".
Ngày 3: lượt L2 đổi Genre của 3 trong 12 file đó.
Ngày 5: hoàn tác L1.
- Album của cả 12 file được đề xuất trở về "Bolero 4". Không có xung đột, vì L2 không đổi Album.
- Genre không bị đụng tới.
- Lịch sử có thêm L3 "Hoàn tác của L1", 12 file.

## Trạng thái & chuyển trạng thái
Mục lịch sử: Bình thường, Đã hoàn tác (toàn bộ), Đã hoàn tác một phần.

## Thông báo lỗi & ràng buộc
| Tình huống | Thông báo |
|---|---|
| File trong nhật ký không còn tồn tại | "File không còn tồn tại" (bỏ qua file đó) |
| Nguồn không khả dụng | "Nguồn chưa kết nối, không thể hoàn tác" |

## Cấu hình ảnh hưởng hành vi
- "Thời gian giữ lịch sử" (mặc định 90 ngày), "Số mục tối đa" (mặc định 200).

## Tiêu chí chấp nhận
### APP-UNDO-V1 — Hoàn tác khôi phục giá trị gốc
- *Kiểm chứng:* APP-UNDO-R1, APP-UNDO-R2, APP-UNDO-R3
- *Điều kiện trước:* Lượt L1 như ví dụ.
- *Các bước:* Lịch sử, chọn L1, Hoàn tác, Áp dụng.
- *Kết quả mong đợi:* 12 file có Album "Bolero 4"; lịch sử có mục "Hoàn tác của L1".

### APP-UNDO-V2 — Trường bị sửa sau đó được bảo vệ
- *Kiểm chứng:* APP-UNDO-R4
- *Điều kiện trước:* L1 đổi Title của file A từ "X" thành "Y". Sau đó L2 đổi Title của A thành "Z".
- *Các bước:* Hoàn tác L1.
- *Kết quả mong đợi:* Title của A được đánh dấu "Đã thay đổi sau lượt này" và mặc định không chọn.
