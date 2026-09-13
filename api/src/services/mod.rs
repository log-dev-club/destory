//! 비즈니스 로직과 DB 접근. 핸들러는 여기 함수를 호출만 한다.

pub mod attachments;
pub mod auth;
pub mod comments;
pub mod discord;
pub mod drafts;
pub mod github;
pub mod posts;
pub mod tags;
pub mod users;

/// ILIKE 패턴 생성. `%`, `_`, `\` 를 이스케이프해 부분 일치 검색에 사용한다.
pub(crate) fn like_pattern(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('%');
    for c in s.chars() {
        if matches!(c, '%' | '_' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('%');
    out
}

/// 바이트 슬라이스를 소문자 hex 문자열로
pub(crate) fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// RFC 3986 비예약 문자를 제외한 모든 바이트를 %XX 로 인코딩 (쿼리 값, Content-Disposition filename* 용)
pub(crate) fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// 문자열을 char 경계에 맞춰 최대 `max_chars` 글자로 자른다
pub(crate) fn truncate_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}
