---
status: draft
app: app
---

# Thư viện (màn hình chính)

Bố cục màn hình chính:
- **Sidebar** (bên trái): Nguồn thư viện, cây thư mục, bộ lọc nhanh.
- **Bảng track** (ở giữa).
- **Inspector** (bên phải): chi tiết và sửa file đang chọn.
- **Thanh hành động** (phía dưới): bộ đếm thay đổi chờ, nút "Xem trước & áp dụng", tiến độ tác vụ nền.

## APP-LIB-OPEN — Thêm nguồn thư viện

### Là gì?
Khai báo thư mục gốc chứa nhạc để app quét và quản lý.

### Khi nào dùng?
Lần đầu dùng app, hoặc khi có thêm ổ hay thư mục nhạc mới.

### Điều kiện trước
Không có.

### Màn hình / giao diện
- Lần đầu mở app: màn hình chào với nút "Thêm thư mục nhạc" và vùng kéo-thả thư mục.
- Sau đó: nút "+" ở mục "Nguồn" trên sidebar.

### Luồng chính
- F1. Nhấn "Thêm thư mục nhạc", chọn thư mục trong hộp chọn của hệ điều hành.
- F2. Hoặc kéo-thả thư mục vào cửa sổ.
- F3. Nguồn xuất hiện trên sidebar ở trạng thái "Chưa quét", rồi quét tự động bắt đầu (APP-LIB-SCAN).

### Quy tắc nghiệp vụ áp dụng
APP-SCAN-R1, APP-SCAN-R7.

### Kiểm tra hợp lệ
| Khi nào kiểm tra | Điều kiện hợp lệ | Thông báo khi không hợp lệ |
|---|---|---|
| Chọn thư mục | Chưa là nguồn | "Thư mục này đã có trong thư viện" |
| Chọn thư mục | Không nằm trong nguồn khác | "Thư mục này đã nằm trong nguồn «tên nguồn»" |

### Trường dữ liệu
Tên nguồn (mặc định là tên thư mục, sửa được), Đường dẫn, Thư mục đích mặc định.

### Trạng thái
Xem trạng thái nguồn ở APP-SCAN.

### Ma trận trạng thái – thao tác
| Trạng thái nguồn | Quét | Sửa tag | Gỡ nguồn |
|---|---|---|---|
| Chưa quét / Đã huỷ | Có | Có (file đã có trong chỉ mục) | Có |
| Đang quét | Huỷ | Có | Không |
| Sẵn sàng | Có | Có | Có |
| Không khả dụng | Thử lại | Không | Có |

### Ma trận quyền – thao tác
Không áp dụng (một người dùng).

### Định dạng số
Không áp dụng.

### Trạng thái rỗng & lỗi
Chưa có nguồn: màn hình chào.

## APP-LIB-SCAN — Quét nguồn

### Là gì?
Đọc tag các file trong nguồn vào chỉ mục.

### Màn hình / giao diện
Thanh tiến độ ở thanh hành động: "Đang quét «nguồn» — n / tổng · x file/giây · còn khoảng t", và nút "Huỷ". Khi xong, thông báo tóm tắt.

### Luồng chính
- F1. Quét tự động sau khi thêm nguồn; hoặc chuột phải vào nguồn và chọn "Quét lại".
- F2. Bảng hiện dần các file đã đọc.
- F3. Khi xong, hiện tóm tắt (APP-SCAN-R9). Nhấn vào "lỗi n" để lọc ra các file lỗi.

### Quy tắc nghiệp vụ áp dụng
APP-SCAN-R2, R3, R4, R5, R6, R7, R9; APP-TAG-R1.

### Kiểm tra hợp lệ
Không có.

### Trường dữ liệu
Không áp dụng.

### Trạng thái
Xem APP-SCAN.

### Ma trận trạng thái – thao tác
Như APP-LIB-OPEN.

### Ma trận quyền – thao tác
Không áp dụng.

### Định dạng số
Số file có phân cách nghìn bằng dấu chấm (20.000). Tốc độ là số nguyên. Thời gian còn lại dạng "2 phút 10 giây".

### Trạng thái rỗng & lỗi
Nguồn không có file hỗ trợ: "Không tìm thấy file nhạc nào trong «nguồn»".

## APP-LIB-BROWSE — Duyệt bảng track

### Là gì?
Bảng tất cả track, đóng vai trò "bảng tính" của app.

### Màn hình / giao diện
- Sidebar có cây thư mục. Chọn một thư mục thì bảng hiện các file trong thư mục đó và các thư mục con.
- Cột mặc định: (dấu thay đổi), Tên file, Track, Title, Artist, Album, Album Artist, Year, Genre, Định dạng, Thời lượng.
- Có thể bật thêm mọi trường của APP-TAG-R2 và APP-TAG-R3.
- Thao tác cột: kéo đổi thứ tự, đổi độ rộng, nhấn tiêu đề để sắp xếp.
- Chọn dòng: nhấp, Shift+nhấp, Cmd+nhấp, Cmd+A.
- Chế độ nhóm "Theo album": các dòng được nhóm dưới tiêu đề album, có ảnh bìa nhỏ.

### Luồng chính
- F1. Chọn thư mục trên cây.
- F2. Sắp xếp hoặc nhóm.
- F3. Chọn một hoặc nhiều dòng; inspector cập nhật theo.

### Quy tắc nghiệp vụ áp dụng
APP-SCAN-R8, APP-STAGE-R4.

### Kiểm tra hợp lệ
Không có.

### Trường dữ liệu
APP-TAG-R2, APP-TAG-R3.

### Trạng thái
Dòng: bình thường, có thay đổi chờ, lỗi đọc, lỗi ghi, xung đột (mỗi loại có biểu tượng riêng).

### Ma trận trạng thái – thao tác
| Trạng thái dòng | Sửa | Áp dụng | Bỏ thay đổi |
|---|---|---|---|
| Bình thường | Có | Không áp dụng | Không áp dụng |
| Có thay đổi chờ | Có | Có | Có |
| Lỗi đọc | Không | Không | Không áp dụng |
| Lỗi ghi / Xung đột | Có | Có (thử lại) | Có |

### Ma trận quyền – thao tác
Không áp dụng.

### Định dạng số
Thời lượng "m:ss" hoặc "h:mm:ss". Dung lượng "12,4 MB".

### Trạng thái rỗng & lỗi
Thư mục không có file: "Thư mục trống".

## APP-LIB-FILTER — Tìm kiếm & bộ lọc nhanh

### Là gì?
Thu hẹp bảng để rà soát. Thay cho việc lọc trên bảng tính.

### Màn hình / giao diện
- Ô tìm kiếm trên thanh công cụ: tìm trên Title, Artist, Album, Album Artist, Tên file. Không phân biệt hoa/thường và dấu tiếng Việt.
- Bộ lọc nhanh trên sidebar, mỗi mục kèm số lượng:
  - "Có thay đổi chờ"
  - "Lỗi" (đọc/ghi/xung đột)
  - "Thiếu Album", "Thiếu Artist", "Thiếu Album Artist", "Thiếu Track", "Thiếu ảnh bìa"
  - "Album không đồng nhất": trong cùng thư mục có nhiều giá trị Album hoặc Album Artist khác nhau
  - "Có khoảng trắng thừa"
  - "Không chuẩn Unicode": chuỗi có dạng tách dấu (APP-TAG-R4)
  - "Giá trị giữ chỗ": có "Unknown Artist", "Unknown Title"… (APP-TAG-R12)
  - "WAV: RIFF INFO lệch" (APP-TAG-R10)
- Lọc theo cột: chuột phải vào tiêu đề cột, chọn "Lọc theo giá trị…".

### Luồng chính
- F1. Nhấn một bộ lọc nhanh; bảng chỉ còn các dòng thoả.
- F2. Kết hợp với ô tìm kiếm (điều kiện VÀ).
- F3. Cmd+A chọn toàn bộ kết quả, rồi sửa hàng loạt.

### Quy tắc nghiệp vụ áp dụng
APP-TAG-R4, APP-STAGE-R9 (phạm vi "dòng đang hiển thị").

### Kiểm tra hợp lệ
Không có.

### Trường dữ liệu
Như APP-LIB-BROWSE.

### Trạng thái
Không áp dụng.

### Ma trận trạng thái – thao tác
Không áp dụng.

### Ma trận quyền – thao tác
Không áp dụng.

### Định dạng số
Số lượng cạnh bộ lọc có phân cách nghìn.

### Trạng thái rỗng & lỗi
Không có kết quả: "Không có track nào khớp" và nút "Xoá bộ lọc".

## APP-LIB-INSPECT — Inspector

### Là gì?
Khung bên phải hiển thị và sửa tag của các dòng đang chọn.

### Màn hình / giao diện
- Chọn 1 file: ảnh bìa, các trường sửa được (APP-TAG-R2), thông tin chỉ đọc (APP-TAG-R3), trạng thái lỗi nếu có.
- Chọn nhiều file: chuyển sang biểu mẫu Sửa nhiều file (APP-EDIT-BATCH).
- Trường có thay đổi chờ hiện giá trị gốc gạch ngang ở bên dưới.

### Quy tắc nghiệp vụ áp dụng
APP-STAGE-R1, APP-STAGE-R4, APP-TAG-R6.

### Trạng thái rỗng & lỗi
Không chọn gì: "Chọn một track để xem chi tiết".
