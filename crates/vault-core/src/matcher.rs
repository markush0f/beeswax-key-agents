use std::path::Path;

use crate::patterns::SecretPattern;
use crate::types::KeyMatch;

pub fn find_matches_in_content_streaming<F>(
    file_path: &Path,
    content: &str,
    patterns: &[SecretPattern],
    hardcoded_by_default: bool,
    on_match: &mut F,
) where
    F: FnMut(KeyMatch),
{
    for (i, line) in content.lines().enumerate() {
        for pattern in patterns {
            for caps in pattern.regex.captures_iter(line) {
                if let Some(matched) = caps.get(1) {
                    let key = matched.as_str();

                    on_match(KeyMatch {
                        file_path: file_path.to_path_buf(),
                        line_number: i + 1,
                        provider: pattern.name.to_string(),
                        key: mask_key(key),
                        hardcoded: hardcoded_by_default || is_hardcoded_in_line(line, key),
                    });
                }
            }
        }
    }
}

fn is_hardcoded_in_line(line: &str, key: &str) -> bool {
    let quoted_double = format!("\"{key}\"");
    let quoted_single = format!("'{key}'");
    let quoted_backtick = format!("`{key}`");

    if line.contains(&quoted_double)
        || line.contains(&quoted_single)
        || line.contains(&quoted_backtick)
    {
        return true;
    }

    let trimmed = line.trim();
    trimmed.contains(key) && (trimmed.contains('=') || trimmed.contains(':'))
}

fn mask_key(val: &str) -> String {
    if val.len() >= 12 {
        format!("{}...{}", &val[..10], &val[val.len() - 4..])
    } else {
        "****".to_string()
    }
}
