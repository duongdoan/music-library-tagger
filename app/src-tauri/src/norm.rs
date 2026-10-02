//! Value normalisation and validation (APP-TAG-R4, R6).
use unicode_normalization::UnicodeNormalization;

/// NFC, collapse internal whitespace, trim (APP-TAG-R4).
pub fn normalize(s: &str) -> String {
    let n: String = s.nfc().collect();
    n.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Error message shown to the user, or None when valid. Empty is always valid (= remove).
pub fn validate(field: &str, v: &str) -> Option<&'static str> {
    if v.is_empty() {
        return None;
    }
    match field {
        "year" if !(v.len() == 4 && v.chars().all(|c| c.is_ascii_digit()) && v >= "1000") => Some("Năm phải gồm 4 chữ số"),
        "track" | "tracktotal" | "disc" | "disctotal" => match v.parse::<u32>() {
            Ok(n) if (1..=999).contains(&n) && !v.starts_with('+') => None,
            _ => Some("Giá trị phải là số nguyên từ 1 đến 999"),
        },
        _ => None,
    }
}

/// Canonical form used to compare a value read back from a file with what was written.
pub fn canonical(field: &str, v: &str) -> String {
    let v = normalize(v);
    if crate::model::NUMERIC.contains(&field) {
        // "03" == "3"; "2019-05-01" year == "2019"
        let digits: String = v.chars().take_while(|c| c.is_ascii_digit()).collect();
        if field == "year" {
            return digits.chars().take(4).collect();
        }
        return digits.trim_start_matches('0').to_string();
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_whitespace_and_unicode() {
        assert_eq!(normalize("  Bolero   Tuyển Chọn  IV "), "Bolero Tuyển Chọn IV");
        let decomposed = "Nhu\u{031b}\u{0300} Quy\u{0300}nh"; // NFD-ish input
        assert_eq!(normalize(decomposed), normalize(&normalize(decomposed)));
        assert_eq!(normalize("Như Quỳnh").len(), "Như Quỳnh".len());
    }

    #[test]
    fn validates_numbers() {
        assert_eq!(validate("year", "2019"), None);
        assert!(validate("year", "19").is_some());
        assert_eq!(validate("track", "12"), None);
        assert!(validate("track", "0").is_some());
        assert!(validate("track", "1000").is_some());
        assert_eq!(validate("title", "anything"), None);
        assert_eq!(validate("track", ""), None);
    }

    #[test]
    fn canonical_numbers() {
        assert_eq!(canonical("track", "03"), "3");
        assert_eq!(canonical("year", "2019-05-01"), "2019");
        assert_eq!(canonical("title", " A  B "), "A B");
    }
}
