---
status: draft
app: app
---

# Thao tác trên file: chuyển thư mục & dọn file thừa

## Bối cảnh nghiệp vụ
Quy trình tổ chức thư viện gồm hai việc:
- **Chuyển thư mục:** album đã chuẩn hoá tag được chuyển từ khu "Staging" (thư mục chứa nhạc chưa xử lý) sang khu đích (ví dụ "Good").
- **Dọn file thừa:** bỏ các file album nguyên khối (.flac/.ape kèm .cue) khi đã có bản tách track.

## Vai trò tham gia
- **Người quản lý thư viện.**

## Quy trình end-to-end
[APP-FILE-MOVE](../../features/app/file-ops.md), [APP-FILE-CLEANUP](../../features/app/file-ops.md).

## Quy tắc & logic tính toán
- **APP-FILEOP-R1. Kế hoạch trước, thực hiện sau.** Mọi thao tác file đều lập danh sách kế hoạch trước. Danh sách có từng mục, nguồn, đích và lý do. Chỉ thực hiện các mục người dùng giữ chọn.
- **APP-FILEOP-R2. Không xoá vĩnh viễn.** File bị bỏ được đưa vào Thùng rác của hệ điều hành.
  - Trên ổ mạng không có Thùng rác: chuyển vào thư mục ".mlt-trash" ở gốc nguồn.
- **APP-FILEOP-R3. Chuyển giữ cấu trúc tương đối.** Đích = thư mục đích + đường dẫn tương đối của thư mục album so với thư mục gốc chuyển. Ví dụ ".../Staging/Bolero/CD1" sang ".../Good/Bolero/CD1".
- **APP-FILEOP-R4. Đơn vị chuyển là thư mục cấp cao nhất.** Nếu cả thư mục cha và thư mục con đều được chọn, chỉ chuyển thư mục cha.
- **APP-FILEOP-R5. Đích đã tồn tại thì không ghi đè.** Mục đó được đánh dấu "Trùng đích" và mặc định bỏ chọn.
- **APP-FILEOP-R6. Không chuyển khi còn thay đổi chờ.** Thư mục chứa file có thay đổi chờ hoặc "Lỗi ghi" bị đánh dấu "Còn thay đổi chưa áp dụng" và không được chọn.
- **APP-FILEOP-R7. Chỉ mục đi theo file.** Sau khi chuyển, chỉ mục và lịch sử cập nhật sang đường dẫn mới. Hoàn tác sau đó vẫn tìm đúng file.
- **APP-FILEOP-R8. Điều kiện dọn file nguyên khối.** File X.flac hoặc X.ape là "thừa" khi **tất cả** điều kiện sau đúng:
  - có X.cue cùng thư mục;
  - X.cue tham chiếu tới chính file đó;
  - cùng thư mục có ít nhất bằng số track trong X.cue các file âm thanh khác (bản tách track).
- **APP-FILEOP-R9. .ape không có .cue không bao giờ bị đề xuất dọn.** Đó có thể là bản duy nhất của album.
- **APP-FILEOP-R10. File .cue mặc định giữ lại.** Có tuỳ chọn "Dọn cả .cue".
- **APP-FILEOP-R11. Thao tác file được ghi vào lịch sử** và hoàn tác được: chuyển ngược lại, hoặc lấy lại từ Thùng rác nếu file còn đó.

## Ví dụ số minh hoạ
Thư mục "Album A": "A.cue" (12 track), "A.flac" 480 MB, và 12 file "01.flac".."12.flac".
- "A.flac" thừa, đề xuất dọn. Tiết kiệm 480 MB.

Thư mục "Album B": "B.cue" (10 track), "B.ape", 7 file tách track.
- Không đề xuất, vì 7 < 10. Lý do: "Bản tách track chưa đủ (7/10)".

Thư mục "Album C": "C.ape", không có .cue.
- Không đề xuất (APP-FILEOP-R9).

## Trạng thái & chuyển trạng thái
Mục kế hoạch: Đề xuất, Đã bỏ chọn, Đã thực hiện, Lỗi.

## Thông báo lỗi & ràng buộc
| Tình huống | Thông báo |
|---|---|
| Đích tồn tại | "Trùng đích" |
| Thư mục còn thay đổi chờ | "Còn thay đổi chưa áp dụng" |
| Thiếu bản tách track | "Bản tách track chưa đủ (n/m)" |
| Không đủ quyền | "Không có quyền di chuyển" |

## Cấu hình ảnh hưởng hành vi
- "Thư mục đích mặc định" cho mỗi nguồn.
- "Dọn cả .cue" (mặc định tắt).

## Tiêu chí chấp nhận
### APP-FILEOP-V1 — Chỉ dọn khi bản tách đủ
- *Kiểm chứng:* APP-FILEOP-R8, APP-FILEOP-R9
- *Điều kiện trước:* Ba thư mục như ví dụ.
- *Các bước:* Dọn file thừa trên nguồn.
- *Kết quả mong đợi:* Kế hoạch chỉ có "A.flac" ở trạng thái Đề xuất. Album B được liệt kê với lý do; Album C không xuất hiện.

### APP-FILEOP-V2 — Chặn chuyển khi còn thay đổi chờ
- *Kiểm chứng:* APP-FILEOP-R6
- *Điều kiện trước:* Thư mục X có 1 thay đổi chờ.
- *Các bước:* Chọn X, Chuyển tới "Good".
- *Kết quả mong đợi:* X có nhãn "Còn thay đổi chưa áp dụng", không chọn được.
