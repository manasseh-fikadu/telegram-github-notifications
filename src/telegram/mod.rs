use reqwest::Client;
use serde_json::json;
use tracing::{info, warn};

use crate::config::RouteConfig;
use crate::github::GitHubEvent;

#[derive(Clone)]
pub struct TelegramClient {
    client: Client,
    bot_token: String,
}

impl TelegramClient {
    pub fn new(bot_token: String) -> Self {
        Self {
            client: Client::new(),
            bot_token,
        }
    }

    pub async fn send_message(&self, chat_id: i64, text: &str) -> anyhow::Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        
        let response = self
            .client
            .post(&url)
            .json(&json!({
                "chat_id": chat_id,
                "text": text,
                "parse_mode": "Markdown",
                "disable_web_page_preview": false
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            anyhow::bail!("Telegram API error: {}", body);
        }

        info!(chat_id = %chat_id, "Message sent to Telegram");
        Ok(())
    }

    pub async fn send_event_notification(
        &self,
        routes: &[RouteConfig],
        event: &GitHubEvent,
    ) -> anyhow::Result<()> {
        let event_key = event.event_key();
        let message = event.format_message();

        for route in routes {
            if !matches_repo(&route.repo_pattern, &event.repo.full_name) {
                continue;
            }

            if !matches_event(&route.events, &event_key, &event.event_type) {
                continue;
            }

            info!(
                repo = %event.repo.full_name,
                chat_id = %route.chat_id,
                event = %event_key,
                "Routing event"
            );

            if let Err(e) = self.send_message(route.chat_id, &message).await {
                warn!(error = %e, chat_id = %route.chat_id, "Failed to send message");
            }
        }

        Ok(())
    }
}

fn matches_repo(pattern: &str, repo_name: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    
    if pattern.contains('*') {
        let prefix = pattern.trim_end_matches('*');
        repo_name.starts_with(prefix)
    } else {
        repo_name == pattern
    }
}

fn matches_event(subscribed: &[String], event_key: &str, event_type: &str) -> bool {
    subscribed.iter().any(|s| {
        s == "*" || s == event_key || s == event_type
    })
}
