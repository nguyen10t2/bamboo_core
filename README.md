# Bamboo Core (Rust)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://github.com/BambooEngine/bamboo-core/actions/workflows/rust.yml/badge.svg)](https://github.com/BambooEngine/bamboo-core/actions)

Một nhân bộ gõ tiếng Việt (IME Core) hiệu năng cao được viết bằng Rust, kế thừa và tối ưu hóa từ phiên bản [bamboo-core](https://github.com/BambooEngine/bamboo-core) gốc bằng Go.

## 💡 Ý tưởng & Nguồn gốc

Dự án này là bản port sang Rust và tối ưu hóa chuyên sâu của bộ nhân gõ tiếng Việt **Bamboo**, ban đầu được phát triển bởi **Luong Thanh Lam** trên ngôn ngữ Go.

Bamboo ra đời với mục tiêu cung cấp một giải pháp gõ tiếng Việt linh hoạt, dựa trên các **quy tắc biến đổi (rule-based transformations)** thay vì hardcode logic. Ý tưởng này cho phép bộ gõ dễ dàng thích nghi với nhiều kiểu gõ khác nhau và hỗ trợ các tính năng hiện đại như gõ tự do, kiểm tra chính tả thông minh.

Triết lý cốt lõi của dự án được kế thừa và lấy cảm hứng từ:
- **[bogo.js](https://github.com/lewtds/bogo.js)**: Dự án tiên phong của **Trung Ngo**, giới thiệu mô hình transformation cho bộ gõ tiếng Việt.
- **GoTiengViet**: Bộ gõ kinh điển của **Tran Ky Nam**, chuẩn mực về sự chính xác và trải nghiệm người dùng.
- **[NexusKey](https://github.com/phatMT97/NexusKey)**: Các kỹ thuật tối ưu hóa mảng tĩnh và state machine từ nhân gõ của **Mai Thanh Phát**.

## 🚀 Cải tiến trong phiên bản Rust

Phiên bản Rust này tập trung vào việc đưa hiệu năng lên mức tối đa để có thể chạy mượt mà trên mọi môi trường từ Desktop đến Web (WASM) và Embedded:
- **Tốc độ vượt trội:** Xử lý một âm tiết phức tạp chỉ trong **~0.47ms**, nhanh hơn đáng kể nhờ chiến lược gõ phím không cấp phát bộ nhớ (Zero-Allocation).
- **Tối ưu Hybrid:** Kết hợp sự linh hoạt của Rule Engine và tốc độ của Static Lookup Tables.
- **An toàn:** Đảm bảo tính đúng đắn của dữ liệu Unicode thông qua hệ thống kiểu mạnh của Rust.

## 📦 Cài đặt

Thêm vào `Cargo.toml` của bạn:

```toml
[dependencies]
bamboo-core = "0.3.0"
```

## 🛠️ Sử dụng nhanh

```rust
use bamboo_core::{Engine, Mode, InputMethod};

fn main() {
    let mut engine = Engine::new(InputMethod::telex());
    engine.process_str("tieengs", Mode::Vietnamese);
    println!("Output: {}", engine.output()); // In ra: "tiếng"
}
```

## 🧩 Delta API cho IME

Cung cấp thông tin cần thiết để cập nhật buffer của IME một cách hiệu quả:

```rust
let (backspaces, _, inserted) = engine.process_key_delta('s', Mode::Vietnamese);
// Kết quả: backspaces = 1 (xóa 'a'), inserted = "á"
```

## 👥 Tác giả & Đóng góp

- **Tác giả bản Rust & Tối ưu hóa:** Dao Trong Nguyen ([@nguyen10t2](https://github.com/nguyen10t2))
- **Tác giả bản gốc (Go):** Luong Thanh Lam ([@lamtq](https://github.com/lamtq))
- **Tư vấn kỹ thuật & Cảm hứng tối ưu:** Mai Thanh Phát ([@phatMT97](https://github.com/phatMT97)) - Tác giả **NexusKey**.

## 📜 Giấy phép

Dự án này được phát hành dưới giấy phép MIT. Xem tệp [LICENSE](LICENSE) để biết thêm chi tiết.
