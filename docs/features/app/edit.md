---
status: draft
app: app
---

# Sửa tag

Mọi thao tác trong module này chỉ tạo **thay đổi chờ**. Không thao tác nào ghi trực tiếp vào file (APP-WRITE-R1).

## APP-EDIT-INLINE — Sửa trong ô

### Là gì?
Sửa trực tiếp một ô trong bảng hoặc một trường trong inspector.

### Luồng chính
- F1. Nhấp đúp vào ô, hoặc chọn ô rồi nhấn Enter, để vào chế độ sửa.
- F2. Nhập giá trị. Enter để lưu tạm và chuyển xuống dòng dưới. Tab để sang ô phải. Esc để huỷ.
- F3. Gõ ký tự bất kỳ khi đang đứng ở một ô để sửa ngay, thay toàn bộ nội dung. Phím mũi tên di chuyển giữa các ô.
- F4. Dán, kéo điền và sửa nhiều ô cùng lúc: xem APP-EDIT-FILL.

### Quy tắc nghiệp vụ áp dụng
APP-STAGE-R1, R2, R3, R4, R6, R13, R14; APP-TAG-R4, R5, R6.

### Kiểm tra hợp lệ
| Khi nào kiểm tra | Điều kiện hợp lệ | Thông báo khi không hợp lệ |
|---|---|---|
| Rời ô Year | APP-TAG-R6 | "Năm phải gồm 4 chữ số" |
| Rời ô Track/Disc | APP-TAG-R6 | "Giá trị phải là số nguyên từ 1 đến 999" |
| Rời ô Track | Track ≤ Track Total | "Track không được lớn hơn Track Total" |

### Trường dữ liệu
APP-TAG-R2, trừ Artwork.

### Trạng thái
Xem APP-STAGE.

### Ma trận trạng thái – thao tác
Như APP-LIB-BROWSE.

### Ma trận quyền – thao tác
Không áp dụng.

### Định dạng số
Track/Disc hiển thị số nguyên, không có số 0 ở đầu.

### Trạng thái rỗng & lỗi
Ô không hợp lệ có viền đỏ và tooltip lỗi. Giá trị không được lưu tạm.

## APP-EDIT-FILL — Vùng ô, kéo điền, sao chép & dán (kiểu Excel)

### Là gì?
Thao tác trên vùng ô của bảng track theo đúng thói quen Excel: chọn vùng, kéo nút điền, Cmd+C/Cmd+V qua lại với Excel/Google Sheets, nhập một giá trị cho nhiều ô.

### Khi nào dùng?
- Chép Album, Album Artist, Genre xuống cả thư mục.
- Điền số track liên tiếp.
- Đưa một cột đã chuẩn bị sẵn trong Excel vào app.

### Điều kiện trước
Có dòng đang hiển thị và sửa được.

### Màn hình / giao diện
- Vùng ô có viền nhấn. Ô hoạt động có viền đậm.
- Góc dưới phải của vùng có nút kéo điền (hình vuông nhỏ, con trỏ dấu cộng).
- Trong lúc kéo, vùng sẽ được điền có viền đứt nét.
- Sau khi điền, thanh hành động hiện "Đã điền n ô" kèm hai nút "Sao chép ô" và "Điền chuỗi".

### Luồng chính
- F1. Kéo chuột qua các ô, hoặc Shift+nhấp, hoặc Shift+mũi tên, để chọn vùng.
- F2. Kéo nút điền xuống hoặc lên. Nhấp đúp nút điền để điền tới cuối thư mục. Giữ Option khi thả để đảo cách điền.
- F3. Bấm "Sao chép ô" hoặc "Điền chuỗi" để đổi cách điền vừa thực hiện.
- F4. Cmd+C để sao chép vùng; Cmd+V để dán từ ô hoạt động.
- F5. Đang sửa một ô mà nhấn Cmd+Enter: ghi giá trị cho cả vùng. Cmd+D: điền xuống. Delete: xoá giá trị cả vùng.

### Quy tắc nghiệp vụ áp dụng
APP-STAGE-R6, R13, R14, R15, R16, R17, R18, R19; APP-TAG-R5, R6.

### Kiểm tra hợp lệ
| Khi nào kiểm tra | Điều kiện hợp lệ | Thông báo khi không hợp lệ |
|---|---|---|
| Điền / dán ô số | APP-TAG-R6 | Ô bị bỏ qua; "Đã điền n ô, bỏ qua m ô" |
| Dán | Vừa vùng còn lại | "Đã dán n ô, bỏ k dòng thừa" |

### Trường dữ liệu
Các cột tag đang hiển thị trong bảng.

### Trạng thái
Không áp dụng.

### Ma trận trạng thái – thao tác
Như APP-LIB-BROWSE (dòng "Lỗi đọc" bị bỏ qua).

### Ma trận quyền – thao tác
Không áp dụng.

### Định dạng số
Chuỗi số là số nguyên, không có số 0 ở đầu.

### Trạng thái rỗng & lỗi
Không có ô nào sửa được trong vùng đích: "Không có ô nào được điền".

## APP-EDIT-BATCH — Sửa nhiều file

### Là gì?
Biểu mẫu sửa cùng lúc nhiều file. Thay cho "kéo giá trị trên Google Sheet" và cho công cụ gán tag cũ.

### Khi nào dùng?
Chuẩn hoá một album hay tuyển tập, hoặc gán Genre cho cả thư mục.

### Màn hình / giao diện
Inspector khi chọn từ 2 dòng trở lên, hoặc hộp thoại (Cmd+I). Mỗi trường có:
- giá trị chung, hoặc "(nhiều giá trị)";
- danh sách các giá trị hiện có kèm số file (ví dụ "Bolero 4 (8) · Bolero IV (4)"); chọn một giá trị để áp cho tất cả;
- lựa chọn hành động "Giữ nguyên" / "Đặt giá trị" / "Xoá".

### Luồng chính
- F1. Chọn nhiều dòng.
- F2. Sửa các trường cần đổi.
- F3. Nhấn "Áp vào n file" để tạo thay đổi chờ.

### Quy tắc nghiệp vụ áp dụng
APP-STAGE-R8, R13, R14; APP-TAG-R4, R5, R6.

### Kiểm tra hợp lệ
Như APP-EDIT-INLINE.

### Trường dữ liệu
APP-TAG-R2.

### Trạng thái
Không áp dụng.

### Ma trận trạng thái – thao tác
Không áp dụng.

### Ma trận quyền – thao tác
Không áp dụng.

### Định dạng số
Số file trong ngoặc, có phân cách nghìn.

### Trạng thái rỗng & lỗi
Có file không sửa được: "Đã bỏ qua n file (lỗi đọc hoặc không khả dụng)".

## APP-EDIT-REPLACE — Tìm & thay

### Là gì?
Hộp thoại (Cmd+Shift+H) tìm và thay chuỗi trong một hoặc nhiều trường. Thay cho công cụ tìm & thay cũ.

### Màn hình / giao diện
Ô "Tìm", ô "Thay bằng", chọn trường, kiểu khớp, phạm vi, các tuỳ chọn (APP-STAGE-R9).
- Danh sách "Mẫu đã lưu" ở đầu hộp thoại: chọn một mẫu để điền sẵn mọi ô. Nút "Lưu mẫu" lưu cấu hình hiện tại.
- Khi chọn kiểu "Biểu thức chính quy", dưới ô "Thay bằng" có dòng gợi ý: "$1, $2… nhóm bắt · $<ten> nhóm đặt tên · $& cả đoạn khớp". Bên dưới là bảng xem trước "File · Trường · Cũ · Mới" với phần thay đổi được tô sáng, cập nhật ngay khi gõ.

### Luồng chính
- F1. Nhập chuỗi tìm và chuỗi thay; xem trước cập nhật ngay.
- F2. Bỏ chọn các dòng không muốn thay.
- F3. Nhấn "Thay n mục" để tạo thay đổi chờ.

### Quy tắc nghiệp vụ áp dụng
APP-STAGE-R9, R13, R14.

### Kiểm tra hợp lệ
| Khi nào kiểm tra | Điều kiện hợp lệ | Thông báo khi không hợp lệ |
|---|---|---|
| Khi gõ (kiểu Biểu thức chính quy) | Cú pháp đúng | "Biểu thức không hợp lệ: «chi tiết»" |
| Nhấn Thay | Ô Tìm không rỗng | (nút bị vô hiệu) |

### Trường dữ liệu
Các trường văn bản của APP-TAG-R2.

### Trạng thái
Không áp dụng.

### Ma trận trạng thái – thao tác
Không áp dụng.

### Ma trận quyền – thao tác
Không áp dụng.

### Định dạng số
Không áp dụng.

### Trạng thái rỗng & lỗi
"Không có file nào khớp".

## APP-EDIT-PATTERN — Tag từ tên file

### Là gì?
Điền tag bằng cách tách tên file hoặc đường dẫn theo mẫu.

### Luồng chính
- F1. Chọn các dòng, mở "Tag từ tên file".
- F2. Nhập mẫu, hoặc chọn từ danh sách mẫu đã dùng gần đây.
- F3. Xem trước bảng kết quả; dòng không khớp được đánh dấu "Không khớp".
- F4. Nhấn "Áp dụng" để tạo thay đổi chờ.

### Quy tắc nghiệp vụ áp dụng
APP-STAGE-R10, R13, R14.

### Kiểm tra hợp lệ
| Khi nào kiểm tra | Điều kiện hợp lệ | Thông báo khi không hợp lệ |
|---|---|---|
| Khi gõ | Có ít nhất một token | "Mẫu phải chứa ít nhất một trường, ví dụ %title%" |

### Trường dữ liệu
Các token của APP-STAGE-R10.

### Trạng thái
Không áp dụng.

### Ma trận trạng thái – thao tác
Không áp dụng.

### Ma trận quyền – thao tác
Không áp dụng.

### Định dạng số
%track% và %disc% nhận cả "03" và "3".

### Trạng thái rỗng & lỗi
Không file nào khớp: nút Áp dụng bị vô hiệu.

## APP-EDIT-NUMBER — Đánh số track

### Luồng chính
- F1. Chọn các dòng, sắp xếp theo thứ tự mong muốn.
- F2. Mở "Đánh số track", nhập số bắt đầu, bật/tắt "Ghi Track Total".
- F3. Xem trước, rồi áp dụng.

### Quy tắc nghiệp vụ áp dụng
APP-STAGE-R11, R13.

## APP-EDIT-CASE — Chuẩn hoá chữ hoa & khoảng trắng

### Luồng chính
- F1. Chọn các dòng, chọn trường, chọn kiểu chữ hoa.
- F2. Xem trước, rồi áp dụng.
- F3. Lệnh "Dọn khoảng trắng & Unicode" áp APP-TAG-R4 lên mọi trường văn bản của các dòng chọn.

### Quy tắc nghiệp vụ áp dụng
APP-STAGE-R12, APP-TAG-R4.

## APP-EDIT-RIFFSYNC — Đồng bộ tag lệch (RIFF INFO của WAV, khoá Album Artist)

### Luồng chính
- F1. Lọc "WAV: RIFF INFO lệch" hoặc "Album Artist lệch giữa các khoá" (hoặc chọn file bất kỳ).
- F2. Chọn "Đồng bộ tag lệch". Mỗi file lệch tạo một thay đổi chờ đặc biệt. Trong Xem trước, mục này hiện là "Ghi lại RIFF INFO theo ID3" hoặc "Ghi Album Artist vào mọi khoá đang dùng («giá trị»)".
- F3. Áp dụng như mọi lượt ghi; có nhật ký và hoàn tác.

### Quy tắc nghiệp vụ áp dụng
APP-TAG-R10, APP-TAG-R13, APP-WRITE-R1..R11, APP-UNDO-R1.

## APP-EDIT-ARTWORK — Ảnh bìa

### Luồng chính
- F1. Kéo-thả ảnh vào ô ảnh bìa ở inspector (một hoặc nhiều file đang chọn).
- F2. Hoặc chọn "Dùng ảnh trong thư mục": app tìm cover.jpg, folder.jpg, front.jpg (không phân biệt hoa/thường) trong thư mục của từng file.
- F3. "Xoá ảnh bìa".
- F4. "Lưu ảnh ra file…" để xuất ảnh bìa hiện có.

### Quy tắc nghiệp vụ áp dụng
APP-TAG-R8, APP-STAGE-R1.

### Kiểm tra hợp lệ
| Khi nào kiểm tra | Điều kiện hợp lệ | Thông báo khi không hợp lệ |
|---|---|---|
| Thả ảnh | JPEG/PNG | "Chỉ hỗ trợ ảnh JPEG hoặc PNG" |
| Thả ảnh | ≤ 10 MB | "Ảnh vượt quá 10 MB" |

## APP-EDIT-REVERT — Bỏ thay đổi & hoàn tác thao tác sửa

### Luồng chính
- F1. Cmd+Z / Cmd+Shift+Z để hoàn tác hoặc làm lại thao tác sửa.
- F2. Chuột phải và chọn "Bỏ thay đổi" cho ô, dòng hoặc vùng chọn.
- F3. Thanh hành động có "Bỏ tất cả" (cần xác nhận "Bỏ n thay đổi chưa áp dụng?").

### Quy tắc nghiệp vụ áp dụng
APP-STAGE-R5, R6, R7.
