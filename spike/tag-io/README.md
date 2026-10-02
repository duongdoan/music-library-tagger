# tag-io spike

Kiểm tra Rust (`lofty`, crate `id3`, cùng module DSF và RIFF INFO tự viết) có đọc/ghi an toàn được thư viện thật không. Kết quả xem tại `docs/architecture/spike-report.md`.

```bash
cargo build --release
# đọc tag
./target/release/tag-io-spike probe <file>...
# chép file vào <out_dir>, ghi giá trị thử, kiểm tra, khôi phục, kiểm tra lại
./target/release/tag-io-spike writetest <out_dir> <file>...
# tốc độ liệt kê thư mục song song
./target/release/tag-io-spike list /Volumes/nas1 16
# tốc độ đọc tag: <listfile> chứa đường dẫn tương đối so với <prefix>
./target/release/tag-io-spike readbench <listfile> <prefix> 600 8 [offset]
```

`writetest` chỉ ghi vào bản sao trong `<out_dir>`, không bao giờ ghi lên file gốc.
