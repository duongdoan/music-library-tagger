---
status: draft
app: app
---

# Quét & chỉ mục thư viện

## Bối cảnh nghiệp vụ
Thư viện có hàng chục nghìn file trên ổ mạng (NAS), nên mỗi lần đọc lại toàn bộ rất chậm. App giữ một **chỉ mục** cục bộ: bản sao tag và thông tin file được lưu trên máy, giúp mở lại tức thì và chỉ đọc lại những file đã thay đổi.

## Vai trò tham gia
- **Người quản lý thư viện.**

## Quy trình end-to-end
1. Thêm thư mục nguồn ([APP-LIB-OPEN](../../features/app/library.md)).
2. Quét ([APP-LIB-SCAN](../../features/app/library.md)).
3. Duyệt, lọc, rà soát ([APP-LIB-BROWSE](../../features/app/library.md), [APP-LIB-FILTER](../../features/app/library.md)).

## Quy tắc & logic tính toán
- **APP-SCAN-R1. Nguồn thư viện.** Người dùng thêm một hoặc nhiều thư mục gốc. Mỗi thư mục gốc được quét đệ quy.
  - Bỏ qua thư mục và file ẩn (tên bắt đầu bằng ".").
  - Bỏ qua file rác hệ thống của macOS (tên bắt đầu bằng "._").
- **APP-SCAN-R2. Quét chạy nền.** Giao diện vẫn thao tác được trong lúc quét. File đã đọc xong hiện dần trong bảng.
- **APP-SCAN-R3. Tiến độ.** Hiển thị "Đã quét n / tổng file", tốc độ (file/giây) và thời gian còn lại ước tính.
  - Tổng được đếm trước bằng một lượt liệt kê nhanh, không đọc tag.
- **APP-SCAN-R4. Huỷ quét.** Huỷ sẽ dừng sau file đang đọc. Các file đã đọc được giữ trong chỉ mục.
- **APP-SCAN-R5. Quét tăng dần.** Ở lần quét sau, chỉ đọc lại file có thời điểm sửa hoặc dung lượng khác với chỉ mục.
  - File mới: thêm vào chỉ mục.
  - File không còn tồn tại: xoá khỏi chỉ mục.
- **APP-SCAN-R6. Lỗi đọc theo từng file.** File thuộc định dạng hỗ trợ nhưng đọc lỗi được đánh dấu "Lỗi đọc" kèm lý do, rồi đếm riêng. Quét vẫn tiếp tục.
- **APP-SCAN-R7. Nguồn không truy cập được** (ví dụ ổ mạng chưa kết nối):
  - nguồn được đánh dấu "Không khả dụng";
  - dữ liệu cũ trong chỉ mục vẫn xem được nhưng không sửa được;
  - app không xoá các file của nguồn đó khỏi chỉ mục.
- **APP-SCAN-R8. Định danh file là đường dẫn đầy đủ.** Mọi so khớp giữa chỉ mục, thay đổi chờ áp dụng và lịch sử đều dùng đường dẫn đầy đủ, không dùng vị trí dòng hay tên file.
- **APP-SCAN-R9. Kết quả quét.** Khi xong, hiển thị tóm tắt: số file đã đọc, mới, thay đổi, đã xoá, lỗi đọc, bị bỏ qua (không hỗ trợ).
- **APP-SCAN-R10. Thư mục loại trừ.** Mỗi nguồn có danh sách thư mục loại trừ. App không duyệt vào các thư mục này, kể cả khi liệt kê. App gợi ý loại trừ các thư mục sao lưu của trình phát nhạc (ví dụ "roon-backup") khi phát hiện chúng.
- **APP-SCAN-R11. Quét lần đầu tiếp tục được.**
  - Quét lần đầu lưu kết quả vào chỉ mục sau mỗi lô file.
  - Thoát app giữa chừng rồi mở lại: quét chạy tiếp từ chỗ dừng, không đọc lại các file đã có trong chỉ mục.
  - Thư mục người dùng đang mở trên cây được ưu tiên quét trước.
- **APP-SCAN-R12. Kiểm tra lại thư mục đang xem.** Khi người dùng mở một thư mục, app liệt kê lại riêng thư mục đó (và các thư mục con), đọc lại các file có thay đổi, rồi cập nhật bảng. Nhờ vậy dữ liệu đang sửa luôn mới, kể cả khi lượt quét toàn bộ gần nhất đã lâu.

## Ví dụ số minh hoạ
**Số đo thật trên thư viện hiện tại** (share nas1 qua SMB, ngày 01/10/2026):
- Tổng 73.111 file, trong đó 54.797 là file âm thanh định dạng hỗ trợ.
- Chỉ riêng lượt liệt kê (chưa đọc tag) mất 212 giây, tức khoảng 345 file/giây.
- Liệt kê kèm đọc ngày sửa và dung lượng từng file (cần cho quét tăng dần, APP-SCAN-R5): khoảng 3,5 phút cho toàn thư viện, khi đã loại trừ thư mục sao lưu Roon (APP-SCAN-R10, bớt khoảng 7.400 file không phải nhạc).
- Đọc tag qua SMB: 15–22 file/giây. Giới hạn nằm ở độ trễ mở file trên NAS, không ở việc phân tích tag.
- Quét lần đầu toàn bộ 54.797 file: khoảng 54.797 / 20 ≈ 2.740 giây, tức 45 phút (dao động 40–65 phút). Vì vậy cần APP-SCAN-R11 và APP-SCAN-R12.

**Ví dụ tính toán.** Nguồn "/Volumes/Music" có 20.000 file hỗ trợ và 1.500 file khác (ảnh, .cue, .log).

Lần quét 1:
- Đọc 20.000 file. Trong đó 7 file lỗi.
- Bỏ qua 1.500 file (không hỗ trợ).
- Với tốc độ 200 file/giây trên ổ mạng: thời gian ≈ 20.000 / 200 = 100 giây.

Lần quét 2, sau khi thêm 120 file và sửa tag 30 file bằng app khác:
- Chỉ đọc lại 150 file. Thời gian ≈ 150 / 200 + thời gian liệt kê ≈ 1 giây + liệt kê.
- Tóm tắt: "mới 120, thay đổi 30, đã xoá 0, lỗi 7".

## Trạng thái & chuyển trạng thái
Trạng thái của nguồn:

| Trạng thái | Nghĩa | Chuyển sang |
|---|---|---|
| Chưa quét | Vừa thêm | Đang quét |
| Đang quét | | Sẵn sàng (xong), Đã huỷ (người dùng huỷ), Không khả dụng (mất kết nối) |
| Sẵn sàng | Chỉ mục khớp lần quét gần nhất | Đang quét |
| Đã huỷ | Chỉ mục chưa đầy đủ | Đang quét |
| Không khả dụng | Không truy cập được thư mục | Đang quét (khi kết nối lại) |

## Thông báo lỗi & ràng buộc
| Tình huống | Thông báo |
|---|---|
| Thư mục đã có trong danh sách nguồn | "Thư mục này đã có trong thư viện" |
| Thư mục nằm trong một nguồn đã có | "Thư mục này đã nằm trong nguồn «tên nguồn»" |
| Nguồn không truy cập được | "Không truy cập được «đường dẫn». Kiểm tra kết nối ổ đĩa." |
| File đọc lỗi | Dòng có biểu tượng lỗi, inspector hiện lý do |

## Cấu hình ảnh hưởng hành vi
- "Quét lại khi mở app": Có/Không (mặc định Không).
- "Định dạng hỗ trợ" (APP-TAG-R1).

## Tiêu chí chấp nhận
### APP-SCAN-V1 — Quét tăng dần
- *Kiểm chứng:* APP-SCAN-R5, APP-SCAN-R9
- *Điều kiện trước:* Nguồn đã quét, có 100 file. Sau đó dùng app khác sửa tag 2 file và xoá 1 file.
- *Các bước:* Quét lại.
- *Kết quả mong đợi:* Tóm tắt "thay đổi 2, đã xoá 1". Bảng hiển thị tag mới của 2 file.

### APP-SCAN-V2 — Lỗi một file không dừng quét
- *Kiểm chứng:* APP-SCAN-R6
- *Điều kiện trước:* Nguồn có 1 file .flac hỏng trong 50 file.
- *Các bước:* Quét.
- *Kết quả mong đợi:* 49 file đọc được, 1 file "Lỗi đọc"; tóm tắt "lỗi 1".
