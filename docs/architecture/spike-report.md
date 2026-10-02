# Báo cáo spike kỹ thuật — 01/10/2026

Mục tiêu: xác nhận Rust đọc/ghi an toàn được thư viện thật, đo tốc độ qua SMB, và kiểm tra bảng kiểu Excel trên Glide Data Grid. Mã nguồn spike ở `spike/tag-io/`.

Nguyên tắc: NAS chỉ được đọc. Mọi phép ghi chạy trên bản sao cục bộ của 14 file mẫu lấy từ `nas1`: FLAC, WAV đủ 4 kiểu bố trí tag, DSF, APE, M4A, AIFF, WMA.

## 1. Đọc / ghi tag

### Kết quả cuối (sau khi sửa)
| Định dạng | Cách ghi | Dữ liệu âm thanh không đổi | Ghi rồi khôi phục: không mất tag nào |
|---|---|---|---|
| FLAC (3 file) | `lofty` `FlacFile` | Có | Có. Chỉ mất các trường rỗng (vd. `COMPOSER=`) |
| DSF (2 file, 50–176 MB) | Module tự viết + crate `id3` | Có | Có (13 và 19 frame) |
| APE | `lofty` tổng quát | Có | Có |
| M4A (2 file) | `lofty` `Mp4File` | Có | Có |
| AIFF | crate `id3` | Có | Có |
| WAV (4 kiểu) | RIFF INFO tự viết + crate `id3` | Có | Có. RIFF INFO được đồng bộ theo ID3 đúng APP-TAG-R10; các trường không đụng tới (kể cả mã hoá cũ) giữ nguyên byte |
| WMA | — | — | `lofty` không đọc được, đúng như dự kiến (APP-TAG-R11) |

- Ghi mất 6–400 ms một file trên ổ cục bộ, ghi qua bản tạm rồi đổi tên (APP-WRITE-R5).
- Mọi file sau khi ghi đều đọc lại được bằng CoreAudio (`afinfo`), thời lượng khớp bản gốc. Băm SHA-256 phần âm thanh trước và sau khi ghi giống hệt.

### Phát hiện quan trọng
1. **API tổng quát của `lofty` làm mất tag khi ghi.** FLAC mất `UPC`, `TOOL NAME`, `ALBUM ARTIST`; WAV mất `TXXX:Album Artist` và `TPOS`. Bắt buộc dùng API riêng cho từng định dạng (`FlacFile`, `Mp4File`) hoặc crate `id3`.
2. **`lofty` bỏ các giá trị RIFF INFO không phải UTF-8.** Thư viện có giá trị mã hoá cũ (ví dụ "Siêu thị…" bị lỗi dấu). Đã viết bộ ghi RIFF INFO riêng (khoảng 100 dòng) giữ nguyên byte các trường không sửa.
3. **`lofty` ghi số track vào `IPRT` thay vì `ITRK`.** `ITRK` phổ biến hơn. Bộ ghi riêng ở điểm 2 xử lý luôn việc này.
4. **File tạm phải giữ đúng đuôi**, vì `lofty` nhận diện định dạng theo đuôi file. Đặt tên dạng `.mlt-tmp.<tên gốc>`.
5. **Album Artist có nhiều cách viết:** `ALBUMARTIST`, `ALBUM ARTIST`, `TXXX:Album Artist`. App phải cập nhật mọi biến thể file đang dùng.
6. **Module DSF tự viết chạy đúng ngay lần đầu**, với 2 file thật 50 MB và 176 MB. Ước lượng 2–4 ngày chỉ còn cần cho phần test mở rộng.

## 2. Tốc độ qua SMB (nas1, chỉ đọc)
| Phép đo | Kết quả |
|---|---|
| `find` tuần tự (chỉ tên file) | 73.111 file / 212 s = 345 file/s |
| Liệt kê song song, 4–16 luồng (chỉ tên file) | 65.635 file / 126–128 s = khoảng 515 file/s. Không tăng sau 4 luồng |
| Liệt kê song song kèm ngày sửa và dung lượng (cần cho quét tăng dần) | 65.635 file / 210 s = 312 file/s |
| Đọc 64 KB đầu file, lần đầu chạm, 1 / 4 / 16 / 32 luồng | 16 / 30 / 47 / 37 file/s |
| Đọc đầu và đuôi file, 16 luồng | FLAC 38 → 14 file/s; WAV 53 → 16 file/s |
| Đọc tag bằng `lofty`, 1 / 4 / 8 / 16 luồng | 10 / 14 / 15 / 15 file/s |
| Đọc tag FLAC bằng bộ đọc tối giản (1 lần đọc 128 KB, bỏ qua khối ảnh), 16 luồng | 22 file/s |

**Kết luận:**
- Phần phân tích tag chỉ tốn 1–3 ms. Thời gian nằm ở độ trễ mở file và dò đĩa trên NAS: mỗi file mất 40–55 ms ở lần chạm đầu.
- Mỗi lần đọc thêm phần đuôi file làm giảm khoảng 3 lần. WAV và DSF bắt buộc đọc đuôi, vì tag nằm sau dữ liệu âm thanh.
- **Quét lần đầu khoảng 45 phút** (dao động 40–65) cho 54.797 file. Quét tăng dần khoảng 3,5 phút cộng thời gian đọc các file đã đổi.
- Thiết kế đã cập nhật trong spec: quét lần đầu tiếp tục được sau khi thoát app, ưu tiên thư mục đang mở (APP-SCAN-R11), và kiểm tra lại thư mục đang xem (APP-SCAN-R12). Bộ đọc nhanh tối giản dùng cho FLAC (72% thư viện); `lofty` dùng khi ghi.
- Hướng tăng tốc về sau: chạy một tác tử quét ngay trên NAS (nếu NAS chạy được Docker) để đọc với tốc độ ổ cục bộ.

## 3. Glide Data Grid 6.0.3
- `onFillPattern(event)` cho biết `patternSource`, `fillDestination` và có `preventDefault()`. App tự cài được logic điền chuỗi số, bỏ qua dòng ẩn/lỗi (APP-STAGE-R16, R17).
- Mặc định: chép lặp vòng theo mẫu nguồn, giống prototype.
- Có sẵn chọn vùng hình chữ nhật, sao chép/dán dạng tab, `onPaste`, `coercePasteValue`, `allowedFillDirections`.
- Sự kiện điền không cho biết phím Option có đang giữ không, nên app tự theo dõi trạng thái phím.
- **Không cần RevoGrid.**

## 4. Chưa kiểm
- **Đổi tên đè file (atomic rename) trên SMB share.** Cần ghi một thư mục thử nghiệm lên NAS, chờ người dùng cho phép.
- **Tốc độ ghi qua SMB.** Ghi qua bản tạm nghĩa là chép cả file (FLAC trung bình 50 MB, DSF 325 MB) qua mạng.
- WMA: không làm (chỉ liệt kê).
