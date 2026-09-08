//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sqids::Sqids;
use std::sync::LazyLock;
use rand::Rng;

static SQIDS: LazyLock<Sqids> = LazyLock::new(|| {
    Sqids::default()
});

pub fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    let random_num: i64 = rng.gen_range(100000..999999);
    SQIDS.encode(&[random_num as u64]).expect("Failed to encode")
}

pub fn generate_code_with_id(id: i64) -> String {
    SQIDS.encode(&[id as u64]).expect("Failed to encode")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code() {
        let code = generate_code();
        println!("Generated code: {}", code);
        assert!(!code.is_empty());
    }

    #[test]
    fn test_generate_code_with_id() {
        let code = generate_code_with_id(12345);
        println!("Generated code with ID: {}", code);
        assert!(!code.is_empty());
    }
}
