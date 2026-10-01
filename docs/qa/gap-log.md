# Gap Log

Gồm hai loại mục: hành vi hiện tại của phần mềm lệch khỏi spec, và các điểm cần quyết định nghiệp vụ.

Phân loại:
- **A**: nghi vấn hành vi (ứng viên lỗi).
- **B**: giới hạn tài liệu, cần quyết định.
- **C**: đã đặc tả nhưng chưa hiện thực.

| ID | Loại | Mô tả | Bằng chứng | Trạng thái | Owner | Quyết định & ngày |
|---|---|---|---|---|---|---|
| GAP-001 | A | Đồng bộ ghép dòng theo vị trí (SYNC-R1). Sắp xếp hoặc lọc bản sao làm lệch dòng, khiến thay đổi bị bỏ qua im lặng. Nếu trang sửa ít dòng hơn trang gốc, công cụ dừng giữa chừng. Nếu hai thư mục có file trùng tên (ví dụ "01.flac"), dòng lệch vẫn bị coi là cùng file. | `update_fix_tag.py:23-26` | Mở | | v2 thay bằng ghép theo đường dẫn đầy đủ (APP) |
| GAP-002 | A | Xuất Excel không đóng workbook, nên nhiều khả năng file Excel không được tạo. | `lib/excel_client.py` (không có `close()`) | Mở | | |
| GAP-003 | A | Xuất Excel: tiêu đề nằm ở dòng 2, dữ liệu từ dòng 3. Tên trang tính dài hơn 31 ký tự gây lỗi. Đường dẫn thư mục thường vượt giới hạn này. | `analyze_tag_excel_v2.py:18-20` | Mở | | |
| GAP-004 | A | Dọn file: mọi tên kết thúc bằng "ape" đều bị đưa vào Thùng rác, kể cả .ape không có .cue (bản duy nhất của album) hoặc tên không có đuôi như "Escape". Không có xem trước. | `tools/remove_full_file.py:26` | Mở | | |
| GAP-005 | A | Chuyển thư mục: khi danh sách có cả thư mục cha và thư mục con, chuyển cha trước khiến đường dẫn con không còn, dẫn tới lỗi. Thư mục gốc lấy từ cấu hình ngoài tham số. | `update_fix_tag.py:69-80` | Mở | | |
| GAP-006 | B | Trang lỗi luôn được tạo, kể cả khi không có lỗi. | `update_fix_tag.py:19` | Mở | | |
| GAP-007 | A | Giao diện desktop: chọn "Output File" gây lỗi. Thư mục mở mặc định không hợp lệ trên macOS. Dùng hộp mở file thay vì hộp lưu file. | `gui.py:63-71` | Mở | | |
| GAP-008 | A | Gán tag nhận đường dẫn từ dòng lệnh nhưng không dùng. | `set_tag.py:6-10, 36` | Mở | | |
| GAP-009 | A | File đọc/ghi lỗi bị bỏ qua im lặng, không có thống kê. Người dùng không biết file nào bị bỏ sót. | Mọi `except: pass` | Mở | | v2: báo cáo lỗi theo file |
| GAP-010 | B | Quét lại cùng thư mục thất bại vì trùng tên trang tính. | `lib/gsheet_client.py:15` | Mở | | |
| GAP-011 | B | Định danh bảng tính và tài khoản dịch vụ được khai báo cố định ở nhiều nơi. Khoá ứng dụng Fshare nằm trong mã. | `gsheet_client.py:13`, `fshare_client.py:31` | Mở | | |
| GAP-012 | A | Trang web chạy quét đồng bộ trong một yêu cầu. Quét thư mục lớn sẽ quá thời gian chờ của trình duyệt. | `main.py:12-15` | Mở | | |
| GAP-013 | B | Đổi tên file và thư mục theo tag (ví dụ "%track% - %title%") có thuộc phạm vi app mới không? Hiện chưa đặc tả. | — | Mở | Product | |
| GAP-014 | B | Tra cứu tag trực tuyến (MusicBrainz, Discogs) và tải ảnh bìa: đưa vào bản sau hay không? | — | Mở | Product | |
| GAP-015 | B | Nhiều nghệ sĩ: giữ một chuỗi có dấu phân cách (APP-TAG-R7) hay hỗ trợ tag đa giá trị thực sự? | — | Mở | Product | |
| GAP-016 | B | Định dạng DSD: chỉ DSF có tag. DFF và ISO SACD không có tag chuẩn nên chưa đưa vào APP-TAG-R1. Cần xác nhận. | Thống kê nas1: 1.033 DSF, không có DFF | Đóng | Product | Chỉ hỗ trợ DSF (thư viện không có DFF), 01/10/2026 |
| GAP-017 | B | Thời gian giữ lịch sử (90 ngày / 200 mục) và dung lượng nhật ký khi lượt áp dụng có ảnh bìa: cần chốt giới hạn. | — | Mở | Product | |
| GAP-018 | B | Ổ mạng không có Thùng rác: dùng thư mục ".mlt-trash" ở gốc nguồn (APP-FILEOP-R2). Cần xác nhận tên và chính sách dọn thư mục này. | — | Mở | Product | |
| GAP-019 | B | Roon không công bố đọc vùng tag nào của WAV. Mẫu 200 file: 91% file có hai vùng lệch nhau, ID3 chứa giá trị đã chỉnh, RIFF INFO chứa giá trị gốc từ nguồn tải. Cần xác nhận trên Roon (file Perfect Crime.wav của Guns N' Roses hiện Artist nào). | Lấy mẫu 200 WAV trên nas1 | Đóng | Product | Roon hiện "Guns N' Roses" (giá trị ID3) cho Perfect Crime.wav. Chốt ID3 là nguồn chuẩn; đồng bộ RIFF INFO phục vụ app khác, không ảnh hưởng Roon. 01/10/2026 |
| GAP-020 | B | Có 30 file WMA. Đề xuất chỉ đọc (APP-TAG-R11), hoặc chuyển đổi sang FLAC ngoài app. | Thống kê nas1 | Mở | Product | |
| GAP-021 | B | Mã hoá khi ghi RIFF INFO: UTF-8 (giữ được dấu tiếng Việt) hay Windows-1252 (tương thích app cũ, mất dấu). Đề xuất UTF-8. | Mẫu nas1 có giá trị mất dấu dạng "Th?o" | Mở | Product | |
