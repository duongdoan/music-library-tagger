---
status: draft
app: app
---

# Mô hình dữ liệu tag

## Bối cảnh nghiệp vụ
App quản lý tag của thư viện nhạc cá nhân, chủ yếu là nhạc Việt và nhạc cổ điển, nhiều định dạng lossless và Hi-Res. Mọi màn hình dùng chung một mô hình tag thống nhất, bất kể định dạng file.

## Vai trò tham gia
- **Người quản lý thư viện**: một người dùng duy nhất trên máy cá nhân. Không có phân quyền.

## Quy trình end-to-end
Áp dụng cho mọi Feature thuộc ứng dụng (APP-LIB-*, APP-EDIT-*, APP-APPLY-*).

## Quy tắc & logic tính toán
- **APP-TAG-R1. Định dạng hỗ trợ.** MP3, FLAC, M4A/AAC/ALAC, OGG/Opus, AIFF, WAV, APE, WavPack, DSF. WMA chỉ liệt kê (APP-TAG-R11).
  - File có đuôi khác được bỏ qua khi quét và không tính là lỗi.
- **APP-TAG-R2. Trường tag sửa được** (nhãn hiển thị):
  - Văn bản: Title, Artist, Album, Album Artist, Composer, Genre, Comment.
  - Số và năm: Year, Track, Track Total, Disc, Disc Total.
  - Cờ và hình: Compilation (Có/Không), Artwork (ảnh bìa trước).
- **APP-TAG-R3. Thông tin chỉ đọc**: Tên file, Thư mục, Định dạng, Thời lượng, Bitrate, Sample rate, Bit depth, Dung lượng, Thời điểm sửa file.
- **APP-TAG-R4. Chuẩn hoá khi lưu tạm** (áp dụng cho mọi giá trị văn bản người dùng nhập):
  - bỏ khoảng trắng đầu và cuối;
  - gộp nhiều khoảng trắng liên tiếp thành một;
  - chuyển về dạng Unicode tổ hợp sẵn (NFC). Ví dụ: "Như Quỳnh" gõ từ bàn phím macOS và từ Windows phải cho ra cùng một chuỗi.
- **APP-TAG-R5. Rỗng nghĩa là xoá.** Lưu một trường văn bản rỗng nghĩa là xoá trường đó khỏi file.
- **APP-TAG-R6. Ràng buộc trường số.**
  - Year: 4 chữ số, từ 1000 đến 9999.
  - Track, Track Total, Disc, Disc Total: số nguyên từ 1 đến 999.
  - Track không được lớn hơn Track Total khi cả hai cùng có giá trị. Disc và Disc Total có ràng buộc tương tự.
- **APP-TAG-R7. Một giá trị cho mỗi trường.** Nhiều nghệ sĩ được ghi trong cùng một chuỗi với dấu phân cách tuỳ người dùng (ví dụ "Như Quỳnh; Mạnh Đình"). App không tách chuỗi.
- **APP-TAG-R8. Artwork.**
  - Chỉ quản lý ảnh bìa trước.
  - Ảnh nhận vào: JPEG hoặc PNG, tối đa 10 MB.
  - Hiển thị kích thước pixel và dung lượng ảnh.
- **APP-TAG-R9. Tag không thuộc danh sách quản lý được giữ nguyên** khi ghi (ví dụ ReplayGain, Lyrics, MusicBrainz ID).
- **APP-TAG-R10. File WAV: ID3 là nguồn chuẩn, RIFF INFO được đồng bộ theo.** File WAV trong thư viện thường có cả vùng ID3 lẫn vùng RIFF INFO (một vùng tag kiểu cũ của WAV), và hai vùng thường lệch nhau.
  - Khi đọc: chỉ dùng ID3. Chỉ khi file không có ID3 mới lấy từ RIFF INFO.
  - Khi ghi: ghi đủ mọi trường vào ID3. Đồng thời ghi RIFF INFO cho các trường vùng này hỗ trợ (Title, Artist, Album, Genre, Year, Track, Comment), mã hoá UTF-8. Trường đó trống ở ID3 thì xoá trong RIFF INFO. Các trường RIFF INFO khác giữ nguyên.
  - File WAV chưa có ID3: tạo mới ID3 khi ghi lần đầu, lấy giá trị ban đầu từ RIFF INFO.
  - Mỗi file WAV có trạng thái "RIFF INFO lệch" khi một trong 7 trường trên khác giữa hai vùng (sau chuẩn hoá APP-TAG-R4).
- **APP-TAG-R11. Định dạng chỉ liệt kê.** File WMA xuất hiện trong bảng với tên file, thư mục và dung lượng, kèm nhãn "Không hỗ trợ sửa tag". App không đọc và không ghi tag của các file này. Chúng được tính vào APP-STAGE-R13.
- **APP-TAG-R12. Giá trị giữ chỗ.** Các giá trị "Unknown Artist", "Unknown Album", "Unknown Title", "Various" (không phân biệt hoa/thường) được coi là thiếu dữ liệu khi lọc rà soát. App không tự xoá các giá trị này.

## Ví dụ số minh hoạ
**WAV (lấy từ thư viện thật).** File "Perfect Crime.wav" có ID3 Artist "Guns N' Roses", RIFF INFO Artist "VINHSTUDIO LOSSLESS WORLD".
- App hiển thị "Guns N' Roses" và đánh dấu file "RIFF INFO lệch".
- Người dùng chạy "Đồng bộ RIFF INFO" rồi áp dụng. Sau đó RIFF INFO Artist cũng là "Guns N' Roses".

Số đo trên 200 file WAV lấy mẫu đều trong nas1 (01/10/2026):
- 181 file có cả hai vùng; trong đó 165 file (91%) lệch ít nhất một trường.
- Số file lệch theo trường: Artist 134, Genre 104, Title 61, Album 15.
- 18 file chỉ có một vùng.

| Người dùng nhập | Sau chuẩn hoá (APP-TAG-R4) |
|---|---|
| "  Bolero   Tuyển Chọn  IV " | "Bolero Tuyển Chọn IV" |
| "Như Quỳnh" (dạng tách dấu) | "Như Quỳnh" (dạng tổ hợp sẵn, cùng chuỗi với bản gõ thường) |

| Track | Track Total | Hợp lệ? |
|---|---|---|
| 3 | 12 | Có |
| 13 | 12 | Không |
| 3 | (trống) | Có |

## Trạng thái & chuyển trạng thái
Xem [APP-STAGE](staging.md).

## Thông báo lỗi & ràng buộc
| Tình huống | Thông báo |
|---|---|
| Year không hợp lệ | "Năm phải gồm 4 chữ số" |
| Track/Disc không phải số nguyên 1–999 | "Giá trị phải là số nguyên từ 1 đến 999" |
| Track > Track Total | "Track không được lớn hơn Track Total" |
| Ảnh sai định dạng | "Chỉ hỗ trợ ảnh JPEG hoặc PNG" |
| Ảnh quá lớn | "Ảnh vượt quá 10 MB" |

## Cấu hình ảnh hưởng hành vi
- "Định dạng hỗ trợ" (Cài đặt): bật/tắt từng định dạng trong APP-TAG-R1.

## Tiêu chí chấp nhận
### APP-TAG-V1 — Chuẩn hoá Unicode
- *Kiểm chứng:* APP-TAG-R4
- *Điều kiện trước:* Hai file có Artist "Như Quỳnh", một ở dạng tách dấu, một ở dạng tổ hợp sẵn.
- *Các bước:* Lọc theo Artist "Như Quỳnh".
- *Kết quả mong đợi:* Cả hai file cùng xuất hiện. Bộ lọc nhanh "Không chuẩn Unicode" chỉ ra file dạng tách dấu.

### APP-TAG-V2 — Chặn Track lớn hơn Track Total
- *Kiểm chứng:* APP-TAG-R6
- *Điều kiện trước:* Một file có Track Total 12.
- *Các bước:* Nhập Track 13.
- *Kết quả mong đợi:* Ô báo lỗi "Track không được lớn hơn Track Total"; thay đổi không được lưu tạm.
