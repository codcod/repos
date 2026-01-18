use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraTicket {
    pub id: String,
    pub key: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub issue_type: String,
    pub assignee: String,
    pub reporter: String,
    pub created: String,
    pub updated: String,
    pub attachments: Vec<JiraAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraAttachment {
    pub filename: String,
    pub content_type: String,
    pub size: Option<u64>,
    pub url: String,
    pub source: String,
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

    pub fn get_ticket(&self, ticket_id: &str) -> Result<JiraTicket> {
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

        self.parse_ticket(ticket_data)
    }

    fn parse_ticket(&self, data: serde_json::Value) -> Result<JiraTicket> {
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
}

pub fn parse_jira_input(input: &str) -> Result<(String, String)> {
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

    // Assume it's just a ticket ID
    let jira_url = env::var("JIRA_URL")
        .context("JIRA_URL not set. Provide full URL or set JIRA_URL environment variable")?;

    Ok((jira_url.trim_end_matches('/').to_string(), input.to_string()))
}
