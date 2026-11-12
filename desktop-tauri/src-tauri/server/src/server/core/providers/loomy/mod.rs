//! Loomy（） ****（ ）。
//!
//! ── this fileWhy this ────────────────────────────────────
//! 、、 to 、 new 、directory、 **** already to 
//! ****（`agent2api-providers` `loomy/`）， not 。
//! in this has `mask_phone` —— is （ to default use ），
//! not up 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：directory unwrap/expect/panic。

/// （`138****8000`）—— default use 。
///
/// ── Why by not is ────────────────────────────────
/// `phone` `POST /api/accounts` request（ `trim` + by ，
/// not ），「11 + 」 is 。 by will 
/// in in → `panic: not a char boundary` → `panic = "abort"` 。
pub fn mask_phone(phone: &str) -> String {
    let chars: Vec<char> = phone.chars().collect();
    if chars.len() < 7 {
        return phone.to_string();
    }
    let head: String = chars[..3].iter().collect();
    let tail: String = chars[chars.len() - 4..].iter().collect();
    format!("{head}****{tail}")
}

#[cfg(test)]
mod tests {
    use super::mask_phone;

    /// ： ASCII `phone` not panic。
    ///
    /// old by `&phone[phone.len() - 4..]`， 16 in `` 
    /// （0xa0） up → panic。 this is 「 by 」 this 。
    #[test]
    fn non_ascii_phone_does_not_panic() {
        assert_eq!(mask_phone("13800008000"), "138****8000");
        // in ： by not will to in 
        let masked = mask_phone("13800008000你");
        assert!(masked.starts_with("138****"));
        // not 
        assert_eq!(mask_phone("123456"), "123456");
    }
}
