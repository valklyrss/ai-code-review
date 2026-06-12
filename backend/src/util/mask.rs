pub fn mask_secret(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    if input.len() <= 6 {
        return "******".to_string();
    }
    format!("{}******{}", &input[..3], &input[input.len() - 3..])
}

pub fn mask_url(input: &str) -> String {
    let mut s = input.to_string();
    for key in ["token=", "access_token=", "password=", "passwd="] {
        if let Some(pos) = s.to_lowercase().find(key) {
            let start = pos + key.len();
            let end = s[start..].find('&').map(|i| start + i).unwrap_or(s.len());
            s.replace_range(start..end, "******");
        }
    }
    s
}

