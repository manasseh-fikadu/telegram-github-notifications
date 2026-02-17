use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize};

#[derive(Debug, Clone)]
pub struct GitHubEvent {
    pub event_type: String,
    pub action: Option<String>,
    pub repo: Repository,
    pub sender: User,
    pub payload: EventPayload,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct Repository {
    pub full_name: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct User {
    pub login: String,
    pub html_url: String,
}

#[derive(Debug, Clone)]
pub enum EventPayload {
    PullRequest(PullRequestPayload),
    Issue(IssuePayload),
    Push(PushPayload),
    WorkflowRun(WorkflowRunPayload),
    Release(ReleasePayload),
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PullRequestPayload {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub state: String,
    pub merged: Option<bool>,
    pub base: BaseRef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BaseRef {
    pub r#ref: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct IssuePayload {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub state: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PushPayload {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub compare: String,
    pub commits: Vec<Commit>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub url: String,
    pub author: CommitAuthor,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct CommitAuthor {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct WorkflowRunPayload {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub head_branch: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReleasePayload {
    pub tag_name: String,
    pub name: Option<String>,
    pub html_url: String,
    pub draft: bool,
    pub prerelease: bool,
}

impl GitHubEvent {
    pub fn parse(event_type: &str, body: &[u8]) -> anyhow::Result<Self> {
        let value = parse_request_body(body)?;

        let action = value
            .get("action")
            .and_then(|v| v.as_str())
            .map(String::from);

        let mut repo: Repository = serde_json::from_value(
            value.get("repository").cloned().unwrap_or_default(),
        )
        .unwrap_or_default();
        if repo.full_name.is_empty() {
            repo.full_name = "unknown/repository".to_string();
        }
        if repo.html_url.is_empty() {
            repo.html_url = "https://github.com".to_string();
        }

        let mut sender: User = serde_json::from_value(
            value.get("sender").cloned().unwrap_or_default(),
        )
        .unwrap_or_default();
        if sender.login.is_empty() {
            sender.login = "unknown".to_string();
        }
        if sender.html_url.is_empty() {
            sender.html_url = "https://github.com".to_string();
        }

        let payload = Self::parse_payload(event_type, &value);

        Ok(GitHubEvent {
            event_type: event_type.to_string(),
            action,
            repo,
            sender,
            payload,
        })
    }

    fn parse_payload(event_type: &str, value: &serde_json::Value) -> EventPayload {
        match event_type {
            "pull_request" => {
                if let Some(pr) = parse_nested(value, Some("pull_request")) {
                    EventPayload::PullRequest(pr)
                } else {
                    EventPayload::Unknown
                }
            }
            "issues" => {
                if let Some(issue) = parse_nested(value, Some("issue")) {
                    EventPayload::Issue(issue)
                } else {
                    EventPayload::Unknown
                }
            }
            "push" => {
                if let Some(push) = parse_nested(value, None) {
                    EventPayload::Push(push)
                } else {
                    EventPayload::Unknown
                }
            }
            "workflow_run" => {
                if let Some(workflow) = parse_nested(value, Some("workflow_run")) {
                    EventPayload::WorkflowRun(workflow)
                } else {
                    EventPayload::Unknown
                }
            }
            "release" => {
                if let Some(release) = parse_nested(value, Some("release")) {
                    EventPayload::Release(release)
                } else {
                    EventPayload::Unknown
                }
            }
            _ => EventPayload::Unknown,
        }
    }

    pub fn event_key(&self) -> String {
        match &self.action {
            Some(action) => format!("{}.{}", self.event_type, action),
            None => self.event_type.clone(),
        }
    }

    pub fn format_message(&self) -> String {
        match &self.payload {
            EventPayload::PullRequest(pr) => {
                let action = self.action.as_deref().unwrap_or("updated");
                let emoji = match action {
                    "opened" => "🆕",
                    "closed" if pr.merged.unwrap_or(false) => "🔀",
                    "closed" => "❌",
                    "reopened" => "🔄",
                    "synchronize" => "📦",
                    _ => "📝",
                };
                format!(
                    "{} *Pull Request {}* [#{}]({})\n`{}` → {}\n_by [{}]({})_",
                    emoji,
                    action,
                    pr.number,
                    pr.html_url,
                    pr.base.r#ref,
                    pr.title,
                    self.sender.login,
                    self.sender.html_url
                )
            }
            EventPayload::Issue(issue) => {
                let action = self.action.as_deref().unwrap_or("updated");
                let emoji = match action {
                    "opened" => "🐛",
                    "closed" => "✅",
                    "reopened" => "🔄",
                    _ => "📋",
                };
                format!(
                    "{} *Issue {}* [#{}]({})\n{}\n_by [{}]({})_",
                    emoji,
                    action,
                    issue.number,
                    issue.html_url,
                    issue.title,
                    self.sender.login,
                    self.sender.html_url
                )
            }
            EventPayload::Push(push) => {
                let branch = push.ref_name.trim_start_matches("refs/heads/");
                let commits = push.commits.len();
                format!(
                    "⬆️ *Push* to `{}`\n[Compare]({}) • {} commit(s)\n_by [{}]({})_",
                    branch, push.compare, commits, self.sender.login, self.sender.html_url
                )
            }
            EventPayload::WorkflowRun(workflow) => {
                let emoji = match workflow.conclusion.as_deref() {
                    Some("success") => "✅",
                    Some("failure") => "❌",
                    Some("cancelled") => "🚫",
                    _ => "⏳",
                };
                format!(
                    "{} *Workflow* `{}`\nBranch: `{}` • Status: {}\n[View Run]({})",
                    emoji,
                    workflow.name,
                    workflow.head_branch,
                    workflow.conclusion.as_deref().unwrap_or(&workflow.status),
                    workflow.html_url
                )
            }
            EventPayload::Release(release) => {
                let emoji = if release.draft {
                    "📝"
                } else if release.prerelease {
                    "🧪"
                } else {
                    "🏷️"
                };
                format!(
                    "{} *Release* `{}`\n{}\n[View Release]({})\n_by [{}]({})_",
                    emoji,
                    release.tag_name,
                    release.name.as_deref().unwrap_or(&release.tag_name),
                    release.html_url,
                    self.sender.login,
                    self.sender.html_url
                )
            }
            EventPayload::Unknown => {
                format!(
                    "📡 *{}* on `{}`\n_by [{}]({})_",
                    self.event_type, self.repo.full_name, self.sender.login, self.sender.html_url
                )
            }
        }
    }
}

fn parse_request_body(body: &[u8]) -> anyhow::Result<serde_json::Value> {
    if let Ok(value) = serde_json::from_slice(body) {
        return Ok(value);
    }

    let form: HashMap<String, String> = serde_urlencoded::from_bytes(body)?;
    let payload = form
        .get("payload")
        .ok_or_else(|| anyhow::anyhow!("missing payload field in form body"))?;
    let value: serde_json::Value = serde_json::from_str(payload)?;
    Ok(value)
}

fn parse_nested<T: DeserializeOwned>(value: &serde_json::Value, field: Option<&str>) -> Option<T> {
    let source = match field {
        Some(key) => value.get(key)?.clone(),
        None => value.clone(),
    };
    serde_json::from_value(source).ok()
}
