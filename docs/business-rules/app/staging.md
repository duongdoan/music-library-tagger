---
status: draft
app: app
---

# Thay đổi chờ áp dụng & các phép sửa hàng loạt

## Bối cảnh nghiệp vụ
App thay thế việc "sửa trên bảng tính". Mọi chỉnh sửa trước hết được giữ ở trạng thái **chờ áp dụng**: chưa ghi vào file, nhưng thấy được trên bảng. Nhờ vậy người dùng có thể sửa hàng nghìn file, rà lại, bỏ bớt, rồi mới ghi một lần.

## Vai trò tham gia
- **Người quản lý thư viện.**

## Quy trình end-to-end
1. Chọn file trong bảng ([APP-LIB-BROWSE](../../features/app/library.md)).
2. Sửa bằng một trong các cách:
   - sửa ô ([APP-EDIT-INLINE](../../features/app/edit.md));
   - sửa nhiều file ([APP-EDIT-BATCH](../../features/app/edit.md));
   - tìm & thay ([APP-EDIT-REPLACE](../../features/app/edit.md));
   - tag từ tên file ([APP-EDIT-PATTERN](../../features/app/edit.md));
   - đánh số track ([APP-EDIT-NUMBER](../../features/app/edit.md));
   - ảnh bìa ([APP-EDIT-ARTWORK](../../features/app/edit.md)).
3. Rà soát và áp dụng ([APP-WRITE](write-apply.md)).

## Quy tắc & logic tính toán
- **APP-STAGE-R1. Thay đổi chờ.** Mỗi thay đổi chờ gồm: file (đường dẫn đầy đủ), trường, giá trị gốc, giá trị mới.
  - Giá trị gốc là giá trị trong chỉ mục tại thời điểm sửa lần đầu.
- **APP-STAGE-R2. Sửa về đúng giá trị gốc thì thay đổi chờ tự huỷ.**
- **APP-STAGE-R3. Sửa nhiều lần cùng một trường** chỉ giữ một thay đổi chờ. Giá trị gốc giữ nguyên, giá trị mới là lần sửa cuối.
- **APP-STAGE-R4. Hiển thị.**
  - Ô có thay đổi chờ được tô màu nhấn.
  - Dòng có thay đổi có dấu chấm ở đầu dòng.
  - Thanh trạng thái hiện "n thay đổi trên m file".
- **APP-STAGE-R5. Thay đổi chờ được lưu bền.** Thoát app (kể cả khi app bị đóng đột ngột) không làm mất thay đổi chờ. Lần mở sau, app hỏi "Khôi phục n thay đổi chưa áp dụng?".
- **APP-STAGE-R6. Hoàn tác trong phiên sửa.** Có hoàn tác (Cmd+Z) và làm lại (Cmd+Shift+Z) cho từng thao tác sửa chưa áp dụng. Một thao tác hàng loạt tính là một bước.
- **APP-STAGE-R7. Bỏ thay đổi** theo ô, theo dòng, theo vùng chọn hoặc tất cả.
- **APP-STAGE-R8. Sửa nhiều file (batch).** Với mỗi trường trong biểu mẫu sửa nhiều file:
  - nếu các file chọn có cùng giá trị: hiện giá trị đó;
  - nếu khác nhau: hiện "(nhiều giá trị)" và mặc định chọn "Giữ nguyên".
  - Người dùng chọn một trong ba hành động cho mỗi trường: "Giữ nguyên", "Đặt giá trị", "Xoá".
  - Chỉ trường có hành động khác "Giữ nguyên" mới sinh thay đổi chờ.
- **APP-STAGE-R9. Tìm & thay.**
  - Phạm vi: các dòng đang chọn, hoặc toàn bộ dòng đang hiển thị sau lọc.
  - Trường: một hoặc nhiều trường văn bản.
  - Kiểu khớp: "Chứa", "Bắt đầu bằng", "Kết thúc bằng", "Toàn bộ giá trị", "Biểu thức chính quy".
  - Tuỳ chọn "Phân biệt hoa/thường" (mặc định tắt) và "Bỏ qua dấu tiếng Việt" (mặc định tắt).
  - Phạm vi thay: "Chứa" và "Biểu thức chính quy" thay mọi lần xuất hiện. "Bắt đầu bằng" chỉ thay phần đầu. "Kết thúc bằng" chỉ thay phần cuối. "Toàn bộ giá trị" thay cả chuỗi.
  - Trước khi xác nhận, hộp thoại xem trước hiện tối đa 200 cặp cũ/mới đầu tiên và tổng số file khớp.
  - Kiểu "Biểu thức chính quy":
    - Hỗ trợ cú pháp thông dụng (giống REGEXREPLACE của Excel): lớp ký tự, lượng từ, neo "^" "$", nhóm bắt, nhóm đặt tên "(?<ten>…)", lookahead/lookbehind.
    - Chuỗi thay được tham chiếu: "$1".."$99" (nhóm bắt), "$<ten>" (nhóm đặt tên), "$&" (toàn bộ đoạn khớp), "$$" (ký tự "$").
    - Biểu thức được kiểm tra ngay khi gõ. Sai cú pháp thì hiện lỗi và không cho thay.
    - Mỗi dòng xem trước tô sáng đoạn khớp trong giá trị cũ.
  - **Mẫu tìm & thay đã lưu**: người dùng lưu bộ (tên, chuỗi tìm, chuỗi thay, trường, kiểu khớp, tuỳ chọn) để dùng lại. App có sẵn các mẫu ở mục ví dụ.
- **APP-STAGE-R10. Tag từ tên file.**
  - Người dùng nhập mẫu gồm các token %title%, %artist%, %album%, %albumartist%, %track%, %disc%, %year%, %genre%, %composer%, %ignore%.
  - Mẫu áp lên tên file (không gồm đuôi), hoặc lên đường dẫn tương đối nếu mẫu chứa "/".
  - File không khớp mẫu không sinh thay đổi và được liệt kê trong xem trước là "Không khớp".
- **APP-STAGE-R11. Đánh số track.**
  - Các file chọn được nhóm theo thư mục.
  - Trong mỗi nhóm, sắp theo thứ tự đang hiển thị, rồi đánh Track từ số bắt đầu (mặc định 1).
  - Tuỳ chọn "Ghi Track Total": bằng số file trong nhóm.
- **APP-STAGE-R12. Chuẩn hoá chữ hoa.** Các kiểu: "Viết Hoa Mỗi Từ", "Viết hoa chữ đầu", "chữ thường", "CHỮ HOA". Áp lên trường được chọn của các file chọn.
- **APP-STAGE-R13. File không sửa được.** File "Lỗi đọc" và file thuộc nguồn "Không khả dụng" không nhận thay đổi chờ. Thao tác hàng loạt bỏ qua các file đó và báo số file bị bỏ qua.
- **APP-STAGE-R14. Mọi giá trị mới đi qua chuẩn hoá và kiểm tra** của APP-TAG-R4 và APP-TAG-R6. File có giá trị không hợp lệ bị loại khỏi thao tác hàng loạt và được liệt kê trong xem trước.
- **APP-STAGE-R15. Vùng ô.** Trong bảng track, người dùng chọn một **vùng ô** hình chữ nhật trên các cột tag sửa được (giống Excel).
  - Cách chọn: kéo chuột, Shift+nhấp, hoặc Shift+phím mũi tên.
  - Ô hoạt động là ô bắt đầu vùng. Phím mũi tên di chuyển ô hoạt động.
  - Các dòng nằm trong vùng ô đồng thời là các dòng đang chọn.
  - Nhấp vào cột Tên file sẽ chọn trọn dòng (mọi cột tag).
  - Vùng ô chỉ tính các dòng đang hiển thị, theo thứ tự đang hiển thị.
- **APP-STAGE-R16. Kéo điền (fill handle).** Góc dưới phải của vùng ô có một nút kéo.
  - Kéo xuống hoặc lên sẽ điền các dòng mở rộng, theo từng cột của vùng nguồn.
  - Nhấp đúp vào nút kéo sẽ điền xuống tới dòng cuối cùng của thư mục chứa ô cuối vùng.
  - Cách điền mặc định:
    - Cột số (Track, Track Total, Disc, Disc Total, Year): nếu vùng nguồn có từ 2 ô và các ô cách đều nhau một bước, app điền tiếp chuỗi số theo bước đó. Các trường hợp còn lại: sao chép.
    - Cột văn bản: sao chép. Nếu vùng nguồn có nhiều ô, các giá trị được lặp lại theo vòng.
  - Giữ phím Option (Alt) khi thả chuột để đảo cách điền (sao chép ↔ điền chuỗi). Với một ô số, điền chuỗi nghĩa là tăng dần 1. Với văn bản kết thúc bằng số (ví dụ "Phần 1"), điền chuỗi nghĩa là tăng con số cuối.
  - Ngay sau khi điền, app hiện lựa chọn "Sao chép ô" / "Điền chuỗi" để đổi cách điền. Đổi cách điền thay thế lần điền vừa rồi, không tạo thêm bước hoàn tác.
- **APP-STAGE-R17. Phạm vi điền.**
  - Chỉ điền vào các dòng đang hiển thị. Dòng bị ẩn do bộ lọc không bao giờ bị điền. Đây là điểm khác Excel, để tránh sửa nhầm dữ liệu không nhìn thấy.
  - Dòng không sửa được (APP-STAGE-R13) bị bỏ qua nhưng vẫn chiếm vị trí trong chuỗi.
  - Giá trị không hợp lệ (ví dụ Track vượt 999) không được điền vào ô đó.
  - App báo "Đã điền n ô, bỏ qua m ô".
- **APP-STAGE-R18. Sao chép & dán vùng.**
  - Cmd+C sao chép vùng ô thành văn bản phân tách bằng tab và xuống dòng. Định dạng này dán được trực tiếp vào Excel hoặc Google Sheets.
  - Cmd+V dán văn bản phân tách bằng tab và xuống dòng (ví dụ sao chép từ Excel), bắt đầu từ ô hoạt động: giá trị thứ i ứng với dòng hiển thị thứ i, cột thứ j ứng với cột hiển thị thứ j.
  - Nếu clipboard chỉ có **một** giá trị và vùng chọn có nhiều ô: giá trị được điền vào mọi ô của vùng.
  - Dữ liệu dán thừa so với số dòng hoặc cột còn lại bị bỏ, kèm báo số lượng.
  - Cột chỉ đọc và dòng không sửa được trong vùng đích bị bỏ qua. Giá trị tương ứng không dịch sang ô khác.
- **APP-STAGE-R19. Nhập cho nhiều ô cùng lúc.**
  - Đang sửa một ô mà nhấn Cmd+Enter: giá trị được ghi vào mọi ô của vùng chọn.
  - Cmd+D: chép dòng đầu của vùng xuống các dòng còn lại (điền xuống).
  - Delete/Backspace: xoá giá trị mọi ô trong vùng (tạo thay đổi chờ xoá trường, APP-TAG-R5).
  - Gõ một ký tự khi đang ở ô hoạt động: bắt đầu sửa ô đó, thay toàn bộ nội dung.
  - Mỗi lệnh kéo điền, dán, Cmd+Enter, Cmd+D, Delete tính là một bước hoàn tác (APP-STAGE-R6).

## Ví dụ số minh hoạ
**Sửa nhiều file.** Chọn 3 file:

| File | Album | Genre |
|---|---|---|
| 01.flac | "Bolero 4" | "Nhạc vàng" |
| 02.flac | "Bolero IV" | "Nhạc vàng" |
| 03.flac | "" | "Bolero" |

Biểu mẫu hiện Album "(nhiều giá trị)" và Genre "(nhiều giá trị)". Người dùng đặt Album "Bolero Tuyển Chọn IV" và để Genre "Giữ nguyên". Kết quả: 3 thay đổi chờ trên trường Album, 0 trên Genre.

**Tìm & thay "Bắt đầu bằng".** Trường Album, tìm "16 / ", thay "":

| Album | Kết quả |
|---|---|
| "16 / Best of Bolero" | "Best of Bolero" |
| "16 / 16 / Tuyển tập" | "16 / Tuyển tập" (chỉ phần đầu; khác bộ script cũ) |
| "Disc 16 / Live" | không khớp |

**Tag từ tên file.** Mẫu "%track% - %artist% - %title%". File "03 - Như Quỳnh - Hai Lối Mộng.flac" cho Track 3, Artist "Như Quỳnh", Title "Hai Lối Mộng". File "Hai Lối Mộng.flac" được liệt kê "Không khớp".

**Đánh số track.** Chọn 5 file: 3 file trong "CD1", 2 file trong "CD2". Bắt đầu từ 1, bật ghi Track Total.
- CD1: 1/3, 2/3, 3/3.
- CD2: 1/2, 2/2.

**Tìm & thay bằng biểu thức chính quy: các mẫu có sẵn**

| Mẫu | Tìm | Thay bằng | Trước | Sau |
|---|---|---|---|---|
| Xoá tiền tố số dạng "16 / " | `^(?:\d+\s*/\s*)+` | (rỗng) | "16 / 16 / Tình Khúc Trịnh Công Sơn" | "Tình Khúc Trịnh Công Sơn" |
| Bỏ số thứ tự ở đầu Title | `^\d{1,3}[\s.\-_]+` | (rỗng) | "06. Đôi Mắt Người Xưa" | "Đôi Mắt Người Xưa" |
| Đảo "Họ, Tên" | `^([^,]+),\s*(.+)$` | `$2 $1` | "Paray, Paul" | "Paul Paray" |
| Xoá phần trong ngoặc vuông | `\s*\[[^\]]*\]` | (rỗng) | "Hạ Trắng [Live 1972]" | "Hạ Trắng" |
| Chuẩn hoá "feat." | `\s*[(\[]?\s*(?:ft\.?\|feat\.?\|featuring)\s+([^)\]]+)[)\]]?` | ` (feat. $1)` | "Hai Lối Mộng ft Mạnh Đình" | "Hai Lối Mộng (feat. Mạnh Đình)" |

**Kéo điền chuỗi số.** Cột Track của 5 dòng hiển thị liên tiếp A, B, C, D, E. A = 1, B = 2. C là file "Lỗi đọc". Chọn vùng A–B, kéo nút điền tới E.
- Vùng nguồn có 2 ô, bước 1, nên app điền chuỗi.
- C bị bỏ qua nhưng vẫn chiếm vị trí số 3.
- D = 4, E = 5. Thông báo: "Đã điền 2 ô, bỏ qua 1 ô".
- Nếu giữ Option khi thả: chuyển sang sao chép, theo vòng 1, 2. Khi đó C ứng với 1 (bị bỏ qua), D = 2, E = 1.

**Kéo điền sao chép.** Ô Album của dòng đầu thư mục là "Bolero Tuyển Chọn IV". Nhấp đúp vào nút điền. Mọi dòng còn lại của thư mục đó (đang hiển thị, sửa được) nhận "Bolero Tuyển Chọn IV".

**Dán từ Excel.** Clipboard chứa 3 dòng × 2 cột (Title, Artist), ô hoạt động là ô Title của dòng 1, bảng chỉ còn 2 dòng hiển thị.
- Dòng 1 và 2 nhận Title, Artist tương ứng.
- Dòng thứ 3 của clipboard bị bỏ. Thông báo: "Đã dán 4 ô, bỏ 1 dòng thừa".

## Trạng thái & chuyển trạng thái
Trạng thái của một trường trên một file:

| Trạng thái | Nghĩa | Chuyển sang |
|---|---|---|
| Đồng bộ | Giá trị bảng = giá trị trong file | Chờ áp dụng (sửa) |
| Chờ áp dụng | Có thay đổi chưa ghi | Đồng bộ (bỏ thay đổi, hoặc sửa về giá trị gốc), Đang ghi |
| Đang ghi | Thuộc lượt áp dụng đang chạy | Đồng bộ (thành công), Lỗi ghi, Xung đột |
| Lỗi ghi | Ghi thất bại, thay đổi vẫn được giữ | Chờ áp dụng (thử lại), Đồng bộ (bỏ) |
| Xung đột | File đã bị thay đổi bên ngoài app sau lần quét | Chờ áp dụng (giữ thay đổi của tôi), Đồng bộ (lấy giá trị trong file) |

## Thông báo lỗi & ràng buộc
| Tình huống | Thông báo |
|---|---|
| Biểu thức chính quy sai cú pháp | "Biểu thức không hợp lệ: «chi tiết»" |
| Tìm & thay không khớp file nào | "Không có file nào khớp" |
| Thao tác hàng loạt gặp file không sửa được | "Đã bỏ qua n file (lỗi đọc hoặc không khả dụng)" |
| Mẫu tên file không chứa token | "Mẫu phải chứa ít nhất một trường, ví dụ %title%" |
| Kéo điền / dán gặp ô không sửa được hoặc giá trị không hợp lệ | "Đã điền n ô, bỏ qua m ô" |
| Dán thừa dòng/cột | "Đã dán n ô, bỏ k dòng thừa" / "bỏ k cột thừa" |
| Dán vào cột chỉ đọc | Ô đó giữ nguyên; tính vào số ô bỏ qua |

## Cấu hình ảnh hưởng hành vi
Không có.

## Tiêu chí chấp nhận
### APP-STAGE-V1 — Sửa nhiều file giữ nguyên trường không chọn
- *Kiểm chứng:* APP-STAGE-R8
- *Điều kiện trước:* 3 file như ví dụ.
- *Các bước:* Mở Sửa nhiều file, đặt Album, để Genre "Giữ nguyên", xác nhận.
- *Kết quả mong đợi:* "3 thay đổi trên 3 file"; Genre của file 03 vẫn là "Bolero".

### APP-STAGE-V2 — Sửa về giá trị gốc tự huỷ thay đổi
- *Kiểm chứng:* APP-STAGE-R2, APP-STAGE-R3
- *Điều kiện trước:* File có Title "A".
- *Các bước:* Sửa Title thành "B", sau đó sửa lại thành "A".
- *Kết quả mong đợi:* Ô không còn tô màu; bộ đếm thay đổi giảm về như trước.

### APP-STAGE-V3 — Thay đổi chờ sống sót khi app bị đóng đột ngột
- *Kiểm chứng:* APP-STAGE-R5
- *Điều kiện trước:* Có 10 thay đổi chờ.
- *Các bước:* Buộc đóng app, mở lại.
- *Kết quả mong đợi:* App hỏi "Khôi phục 10 thay đổi chưa áp dụng?"; chọn Khôi phục thì 10 ô được tô màu như trước.

### APP-STAGE-V4 — Đánh số track theo thư mục
- *Kiểm chứng:* APP-STAGE-R11
- *Điều kiện trước:* Như ví dụ đánh số track.
- *Các bước:* Chọn 5 file, Đánh số track, bắt đầu 1, bật Track Total.
- *Kết quả mong đợi:* CD1 1/3..3/3; CD2 1/2..2/2.

### APP-STAGE-V5 — Kéo điền chuỗi số, bỏ qua file lỗi
- *Kiểm chứng:* APP-STAGE-R16, APP-STAGE-R17
- *Điều kiện trước:* Như ví dụ "Kéo điền chuỗi số".
- *Các bước:* Chọn ô Track của A và B, kéo nút điền tới E.
- *Kết quả mong đợi:* D = 4, E = 5; C không đổi; thông báo "Đã điền 2 ô, bỏ qua 1 ô". Một lần Cmd+Z đưa D và E về giá trị trước.

### APP-STAGE-V6 — Không điền vào dòng bị lọc ẩn
- *Kiểm chứng:* APP-STAGE-R17
- *Điều kiện trước:* Thư mục 6 file. Bộ lọc "Thiếu Album Artist" chỉ hiện 3 file.
- *Các bước:* Nhập Album Artist ở dòng đầu, nhấp đúp nút điền.
- *Kết quả mong đợi:* Chỉ 3 file đang hiển thị nhận giá trị; 3 file bị ẩn không có thay đổi chờ.

### APP-STAGE-V7 — Dán vùng từ Excel
- *Kiểm chứng:* APP-STAGE-R18
- *Điều kiện trước:* Sao chép 3 dòng × 2 cột từ Excel; bảng có 2 dòng hiển thị.
- *Các bước:* Chọn ô Title dòng 1, nhấn Cmd+V.
- *Kết quả mong đợi:* 4 thay đổi chờ; thông báo "Đã dán 4 ô, bỏ 1 dòng thừa".

### APP-STAGE-V8 — Thay bằng nhóm bắt
- *Kiểm chứng:* APP-STAGE-R9
- *Điều kiện trước:* File có Artist "Paray, Paul".
- *Các bước:* Tìm & thay, kiểu "Biểu thức chính quy", tìm `^([^,]+),\s*(.+)$`, thay `$2 $1`.
- *Kết quả mong đợi:* Xem trước "Paray, Paul" → "Paul Paray"; xác nhận thì tạo 1 thay đổi chờ.
