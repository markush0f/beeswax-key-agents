pub fn spinner_ascii(tick: u64) -> &'static str {
    const FRAMES: [&str; 4] = ["-", "\\", "|", "/"];
    FRAMES[(tick as usize) % FRAMES.len()]
}

pub fn elide_middle(input: &str, max_len: usize) -> String {
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    if len <= max_len {
        return input.to_string();
    }
    if max_len <= 3 {
        return ".".repeat(max_len);
    }

    let head = (max_len - 3) / 2;
    let tail = (max_len - 3) - head;
    let start: String = chars[..head].iter().collect();
    let end: String = chars[len - tail..].iter().collect();
    format!("{start}...{end}")
}
