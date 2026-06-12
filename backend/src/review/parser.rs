use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AiReviewResult {
    pub summary: Option<String>,
    #[serde(default)]
    pub issues: Vec<AiIssue>,
}

#[derive(Debug, Deserialize)]
pub struct AiIssue {
    pub level: String,
    #[serde(rename = "type")]
    pub issue_type: Option<String>,
    pub line: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub suggestion: Option<String>,
    #[serde(rename = "needEmail", default)]
    pub need_email: bool,
}

pub fn parse_ai_json(text: &str) -> Result<AiReviewResult, serde_json::Error> {
    let trimmed = text.trim();
    if let Some(json) = extract_code_block(trimmed) {
        return serde_json::from_str(&json);
    }
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        return serde_json::from_str(&trimmed[start..=end]);
    }
    serde_json::from_str(trimmed)
}

fn extract_code_block(text: &str) -> Option<String> {
    let start = text.find("```")?;
    let after = &text[start + 3..];
    let content_start = after.find('\n').map(|i| i + 1).unwrap_or(0);
    let rest = &after[content_start..];
    let end = rest.find("```")?;
    Some(rest[..end].trim().to_string())
}

