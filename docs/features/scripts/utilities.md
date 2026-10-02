---
status: as-built
app: scripts
---

# Tiện ích phụ (ngoài phạm vi quản lý tag)

## SCR-UTIL-CRAWL — Thu thập danh sách bài viết từ blog

### Là gì?
Công cụ duyệt một trang danh mục blog cùng các trang "bài cũ hơn" (tối đa 1000 trang). Công cụ ghi tên và liên kết của từng bài viết ra một trang tính Google mới.

### Luồng chính
- F1. Mở trang danh mục khai báo sẵn.
- F2. Lấy tiêu đề và liên kết của từng bài, ghi theo lô 100 dòng, tiêu đề cột "Name", "Link".
- F3. Theo liên kết "bài cũ hơn", chờ 5 giây giữa hai trang. Gặp lỗi: chờ 1 giây rồi thử lại cùng trang.
- F4. Hết trang: in "No more posts".

## SCR-UTIL-FSHARE — Lấy danh sách file trong thư mục Fshare

### Là gì?
Công cụ đăng nhập Fshare bằng tài khoản trong file cấu hình cục bộ, rồi in danh sách file của một thư mục chia sẻ (tối đa 60 mục một trang).

### Luồng chính
- F1. Đăng nhập. Sai thông tin: dừng với thông báo từ Fshare.
- F2. In danh sách file của thư mục khai báo sẵn.
