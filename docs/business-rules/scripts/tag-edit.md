---
status: as-built
app: scripts
---

# Sửa tag hàng loạt trực tiếp trên file (bộ công cụ script)

## Bối cảnh nghiệp vụ
Một số chỉnh sửa áp dụng đồng loạt cho cả một thư mục, không cần đi qua bảng tính. Ví dụ: gán cùng tên album cho một tuyển tập, hoặc xoá một tiền tố rác trong tên album.

## Vai trò tham gia
- **Người quản lý thư viện.**

## Quy trình end-to-end
- Gán tag: [SCR-EDIT-SET](../../features/scripts/batch-edit.md).
- Tìm & thay: [SCR-EDIT-REPLACE](../../features/scripts/batch-edit.md).

## Quy tắc & logic tính toán
- **EDIT-R1. Gán tag ghi đè ba trường.** Mọi file đọc được tag trong thư mục (đệ quy) được ghi Album, Album Artist, Genre bằng giá trị nhập vào. Giá trị cũ bị thay hoàn toàn. Các trường khác giữ nguyên.
- **EDIT-R2. Gán tag luôn đếm trước** tổng số file và in "Processed n/tổng" sau mỗi file.
- **EDIT-R3. Tìm & thay tác động trên đúng một trường** do người dùng chọn (ví dụ Album).
- **EDIT-R4. Điều kiện khớp là "bắt đầu bằng".** Một file được tính là "tìm thấy" khi giá trị hiện tại của trường bắt đầu bằng chuỗi cần tìm. Phân biệt chữ hoa/thường.
- **EDIT-R5. Thay thế mọi lần xuất hiện.** Với file khớp, giá trị mới bằng giá trị cũ sau khi thay **mọi** lần xuất hiện của chuỗi cần tìm (không chỉ phần đầu).
- **EDIT-R6. Chế độ xem trước.** Khi tắt "thực hiện thay", công cụ chỉ in từng cặp "giá trị cũ  ====>  giá trị mới" và không ghi file.
- **EDIT-R7. Không sao lưu, không hoàn tác.** Ghi trực tiếp vào file; không lưu giá trị cũ.
- **EDIT-R8. Lỗi ghi từng file bị bỏ qua im lặng**, công cụ tiếp tục với file kế tiếp.

## Ví dụ số minh hoạ
Tìm & thay trên trường Album, chuỗi tìm "16 / ", thay bằng chuỗi rỗng:

| Album hiện tại | Khớp? | Album mới |
|---|---|---|
| "16 / Best of Bolero" | Có | "Best of Bolero" |
| "16 / 16 / Tuyển tập" | Có | "Tuyển tập" (thay cả hai lần) |
| "Disc 16 / Live" | Không | (giữ nguyên) |

Kết quả: tìm thấy 2, đã thay 2.

## Trạng thái & chuyển trạng thái
Không áp dụng.

## Thông báo lỗi & ràng buộc
| Tình huống | Hành vi / thông báo |
|---|---|
| Kết thúc tìm & thay | "Finished checking, found n" và "Finished replacing, replaced m" |
| Kết thúc gán tag | "Finished setting tags" |
| Không ghi được một file | Bỏ qua, không thông báo (EDIT-R8) |

## Cấu hình ảnh hưởng hành vi
- Thư mục, trường, chuỗi tìm, chuỗi thay, bật/tắt thực hiện thay, giá trị gán: khai báo cố định trong công cụ.

## Tiêu chí chấp nhận
### EDIT-V1 — Tìm & thay chỉ tác động file bắt đầu bằng chuỗi tìm
- *Kiểm chứng:* EDIT-R4, EDIT-R5
- *Điều kiện trước:* 3 file với Album như bảng ví dụ.
- *Các bước:* Chạy tìm & thay với chuỗi tìm "16 / ", thay "", bật thực hiện thay.
- *Kết quả mong đợi:* Album của 3 file đúng như cột "Album mới"; thông báo "found 2", "replaced 2".

### EDIT-V2 — Xem trước không ghi file
- *Kiểm chứng:* EDIT-R6
- *Điều kiện trước:* Như EDIT-V1.
- *Các bước:* Chạy với tắt thực hiện thay.
- *Kết quả mong đợi:* In 2 cặp cũ/mới; tag trong file không đổi; "replaced 0".
