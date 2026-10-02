---
status: as-built
app: scripts
---

# Dọn file trùng định dạng (bộ công cụ script)

## Bối cảnh nghiệp vụ
Album lossless thường tải về dưới dạng một file lớn (.flac hoặc .ape) kèm file .cue chia track. Khi đã có bản tách track, file lớn trở thành thừa và chiếm dung lượng.

## Vai trò tham gia
- **Người quản lý thư viện.**

## Quy trình end-to-end
[SCR-CLEAN-CUEAPE](../../features/scripts/file-cleanup.md).

## Quy tắc & logic tính toán
- **CLEAN-R1. Theo cặp .cue.** Với mỗi file kết thúc bằng ".cue", công cụ đưa vào Thùng rác file cùng tên có đuôi ".flac" và file cùng tên có đuôi ".ape". File .cue được giữ lại.
- **CLEAN-R2. Mọi file kết thúc bằng "ape" đều bị đưa vào Thùng rác**, kể cả không có .cue đi kèm.
- **CLEAN-R3. Đưa vào Thùng rác của hệ điều hành**, không xoá vĩnh viễn.
- **CLEAN-R4. Không có chế độ xem trước.** Chạy là thực hiện ngay.
- **CLEAN-R5. Lỗi từng file được in ra và bỏ qua.** Ví dụ: không tìm thấy file .flac tương ứng.

## Ví dụ số minh hoạ
Thư mục album có: "CD1.cue", "CD1.flac", "CD2.ape" (không có CD2.cue), "cover.jpg".
- "CD1.flac": vào Thùng rác (CLEAN-R1).
- "CD1.ape": không tồn tại, in lỗi và bỏ qua.
- "CD2.ape": vào Thùng rác (CLEAN-R2).
- "CD1.cue", "cover.jpg": giữ nguyên.

## Trạng thái & chuyển trạng thái
Không áp dụng.

## Thông báo lỗi & ràng buộc
| Tình huống | Hành vi / thông báo |
|---|---|
| Bắt đầu | "Analyzing" |
| File cần dọn không tồn tại | In nội dung lỗi, tiếp tục |
| Kết thúc | "Finished analyzing, processed n", sau đó "DONE" |

## Cấu hình ảnh hưởng hành vi
- Thư mục cần dọn: khai báo cố định trong công cụ.

## Tiêu chí chấp nhận
### CLEAN-V1 — Dọn theo cặp .cue
- *Kiểm chứng:* CLEAN-R1, CLEAN-R3
- *Điều kiện trước:* Thư mục chứa "A.cue" và "A.flac".
- *Các bước:* Chạy công cụ dọn.
- *Kết quả mong đợi:* "A.flac" nằm trong Thùng rác; "A.cue" còn nguyên.
