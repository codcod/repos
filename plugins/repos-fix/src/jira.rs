use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraTicket {
    pub id: String,
    pub key: String,
    pub title: String,
    pub description: String,
    pub labels: Vec<String>,
    pub status: String,
    pub priority: String,
    pub issue_type: String,
    pub assignee: String,
    pub reporter: String,
    pub created: String,
    pub updated: String,
    pub attachments: Vec<JiraAttachment>,
    pub comments: Vec<JiraComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraAttachment {
    pub filename: String,
    pub content_type: String,
    pub size: Option<u64>,
    pub url: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraComment {
    pub author: String,
    pub created: String,
    pub body: String,
}

pub struct JiraClient {
    client: Client,
    base_url: String,
}

impl JiraClient {
    pub fn with_base_url(base_url: String) -> Result<Self> {
        let base_url = base_url.trim_end_matches('/').to_string();

        let username =
            env::var("JIRA_USERNAME").context("JIRA_USERNAME environment variable not set")?;

        let token =
            env::var("JIRA_API_TOKEN").context("JIRA_API_TOKEN environment variable not set")?;

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Add Basic Auth header manually
        let auth_value = format!("{}:{}", username, token);
        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            auth_value.as_bytes(),
        );
        let auth_header = HeaderValue::from_str(&format!("Basic {}", encoded))
            .context("Failed to create auth header")?;
        headers.insert(reqwest::header::AUTHORIZATION, auth_header);

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, base_url })
    }

    pub fn get_ticket(&self, ticket_id: &str, num_comments: usize) -> Result<JiraTicket> {
        let url = format!("{}/rest/api/3/issue/{}", self.base_url, ticket_id);

        let response = self
            .client
            .get(&url)
            .query(&[("expand", "renderedFields,attachments,comments")])
            .send()
            .context("Failed to fetch JIRA ticket")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            anyhow::bail!("JIRA API error ({}): {}", status, error_text);
        }

        let ticket_data: serde_json::Value =
            response.json().context("Failed to parse JIRA response")?;

        self.parse_ticket(ticket_data, num_comments)
    }

    fn parse_ticket(&self, data: serde_json::Value, num_comments: usize) -> Result<JiraTicket> {
        Self::parse_ticket_data(data, num_comments)
    }

    fn parse_ticket_data(data: serde_json::Value, num_comments: usize) -> Result<JiraTicket> {
        let fields = data
            .get("fields")
            .context("Missing 'fields' in JIRA response")?;

        let rendered_fields = data.get("renderedFields");

        let description = if let Some(rendered) = rendered_fields {
            rendered
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
        } else {
            fields
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
        };

        // Clean HTML from description
        let description = html2text::from_read(description.as_bytes(), 80);

        let labels = fields
            .get("labels")
            .and_then(|labels| labels.as_array())
            .map(|labels| {
                labels
                    .iter()
                    .filter_map(|label| label.as_str())
                    .map(|label| label.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let comments = Self::parse_comments(fields, rendered_fields, num_comments);

        let mut attachments = Vec::new();

        // Parse direct JIRA attachments
        if let Some(attachment_array) = fields.get("attachment").and_then(|a| a.as_array()) {
            for attachment in attachment_array {
                if let Some(att) = Self::parse_attachment(attachment) {
                    attachments.push(att);
                }
            }
        }

        Ok(JiraTicket {
            id: data
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("")
                .to_string(),
            key: data
                .get("key")
                .and_then(|k| k.as_str())
                .unwrap_or("")
                .to_string(),
            title: fields
                .get("summary")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string(),
            description,
            labels,
            status: fields
                .get("status")
                .and_then(|s| s.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string(),
            priority: fields
                .get("priority")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string(),
            issue_type: fields
                .get("issuetype")
                .and_then(|i| i.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string(),
            assignee: fields
                .get("assignee")
                .and_then(|a| a.get("displayName"))
                .and_then(|n| n.as_str())
                .unwrap_or("Unassigned")
                .to_string(),
            reporter: fields
                .get("reporter")
                .and_then(|r| r.get("displayName"))
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string(),
            created: fields
                .get("created")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string(),
            updated: fields
                .get("updated")
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .to_string(),
            attachments,
            comments,
        })
    }

    fn parse_attachment(data: &serde_json::Value) -> Option<JiraAttachment> {
        Some(JiraAttachment {
            filename: data
                .get("filename")
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string(),
            content_type: data
                .get("mimeType")
                .and_then(|m| m.as_str())
                .unwrap_or("")
                .to_string(),
            size: data.get("size").and_then(|s| s.as_u64()),
            url: data
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string(),
            source: "jira".to_string(),
        })
    }

    fn parse_comments(
        fields: &serde_json::Value,
        rendered_fields: Option<&serde_json::Value>,
        num_comments: usize,
    ) -> Vec<JiraComment> {
        let Some(comment_array) = fields
            .get("comment")
            .and_then(|comment| comment.get("comments"))
            .and_then(|comments| comments.as_array())
        else {
            return Vec::new();
        };

        let rendered_comments = rendered_fields
            .and_then(|rendered| rendered.get("comment"))
            .and_then(|comment| comment.get("comments"))
            .and_then(|comments| comments.as_array());

        if num_comments == 0 {
            return Vec::new();
        }

        let start_index = comment_array.len().saturating_sub(num_comments);
        comment_array
            .iter()
            .enumerate()
            .skip(start_index)
            .filter_map(|(index, comment)| {
                let rendered_body = rendered_comments
                    .and_then(|comments| comments.get(index))
                    .and_then(|comment| comment.get("body"))
                    .and_then(|body| body.as_str());

                let raw_body = rendered_body
                    .or_else(|| comment.get("body").and_then(|body| body.as_str()))
                    .unwrap_or("");

                let body = html2text::from_read(raw_body.as_bytes(), 80)
                    .trim()
                    .to_string();
                if body.is_empty() {
                    return None;
                }

                let author = comment
                    .get("author")
                    .and_then(|author| author.get("displayName"))
                    .and_then(|name| name.as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let created = comment
                    .get("created")
                    .and_then(|created| created.as_str())
                    .unwrap_or("")
                    .to_string();

                Some(JiraComment {
                    author,
                    created,
                    body,
                })
            })
            .collect()
    }
}

pub fn parse_jira_input(input: &str) -> Result<(String, String)> {
    let jira_url = env::var("JIRA_URL")
        .context("JIRA_URL not set. Provide full URL or set JIRA_URL environment variable")?;
    parse_jira_input_with_base_url(input, Some(jira_url.as_str()))
}

fn parse_jira_input_with_base_url(input: &str, jira_url: Option<&str>) -> Result<(String, String)> {
    let input = input.trim();
    if input.is_empty() {
        anyhow::bail!("JIRA ticket input cannot be empty");
    }

    // Check if it's a full URL
    if input.starts_with("http") {
        let url = url::Url::parse(input).context("Invalid JIRA URL")?;

        let base_url = format!(
            "{}://{}",
            url.scheme(),
            url.host_str().context("Invalid host in URL")?
        );

        // Extract ticket ID from path like /browse/MAINT-1234
        let path = url.path();
        if let Some(ticket_id) = path.strip_prefix("/browse/") {
            return Ok((base_url, ticket_id.to_string()));
        }

        anyhow::bail!("Could not extract ticket ID from URL: {}", input);
    }

    let jira_url = jira_url
        .context("JIRA_URL not set. Provide full URL or set JIRA_URL environment variable")?;

    Ok((
        jira_url.trim_end_matches('/').to_string(),
        input.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_jira_input_with_url() {
        let result = parse_jira_input("https://company.atlassian.net/browse/MAINT-1234").unwrap();
        assert_eq!(result.0, "https://company.atlassian.net");
        assert_eq!(result.1, "MAINT-1234");
    }

    #[test]
    fn parse_jira_input_with_ticket_id_uses_env() {
        let result =
            parse_jira_input_with_base_url("MAINT-5678", Some("https://company.atlassian.net/"))
                .unwrap();
        assert_eq!(result.0, "https://company.atlassian.net");
        assert_eq!(result.1, "MAINT-5678");
    }

    #[test]
    fn parse_jira_input_rejects_empty() {
        assert!(parse_jira_input("  ").is_err());
    }

    #[test]
    fn parse_ticket_prefers_rendered_fields_and_parses_attachments() {
        let data = json!({
            "id": "1001",
            "key": "MAINT-1",
            "fields": {
                "summary": "Fix bug",
                "description": "Raw <b>desc</b>",
                "labels": ["security", "backend"],
                "status": { "name": "Open" },
                "priority": { "name": "P1" },
                "issuetype": { "name": "Bug" },
                "assignee": { "displayName": "Ada" },
                "reporter": { "displayName": "Bob" },
                "created": "2024-01-01",
                "updated": "2024-01-02",
                "attachment": [
                    {
                        "filename": "log.txt",
                        "mimeType": "text/plain",
                        "size": 123,
                        "content": "https://jira/att/1"
                    }
                ],
                "comment": {
                    "comments": [
                        {
                            "author": { "displayName": "Eve" },
                            "created": "2024-01-03",
                            "body": "Raw <p>body</p>"
                        }
                    ]
                }
            },
            "renderedFields": {
                "description": "<p>Rendered <strong>desc</strong></p>",
                "comment": {
                    "comments": [
                        { "body": "<p>Rendered comment</p>" }
                    ]
                }
            }
        });

        let ticket = JiraClient::parse_ticket_data(data, 1).unwrap();

        assert_eq!(ticket.key, "MAINT-1");
        let description = ticket.description.to_lowercase();
        assert!(description.contains("rendered"));
        assert!(!description.contains("raw"));
        assert_eq!(ticket.attachments.len(), 1);
        assert_eq!(ticket.attachments[0].filename, "log.txt");
        assert_eq!(ticket.attachments[0].source, "jira");
        assert_eq!(ticket.comments.len(), 1);
        assert!(ticket.comments[0].body.contains("Rendered comment"));
    }

    #[test]
    fn parse_comments_respects_limit() {
        let fields = json!({
            "comment": {
                "comments": [
                    { "author": { "displayName": "A" }, "created": "2024-01-01", "body": "first" },
                    { "author": { "displayName": "B" }, "created": "2024-01-02", "body": "second" }
                ]
            }
        });
        let rendered = json!({
            "comment": {
                "comments": [
                    { "body": "first" },
                    { "body": "second" }
                ]
            }
        });

        let comments = JiraClient::parse_comments(&fields, Some(&rendered), 1);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].author, "B");
        assert_eq!(comments[0].body, "second");
    }
}
