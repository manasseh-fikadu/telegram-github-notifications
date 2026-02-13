# gh-telegram-forwarder

GitHub webhook → Telegram notification forwarder written in Rust.

## Setup

1. Create a Telegram bot via [@BotFather](https://t.me/botfather) and get the token
2. Copy `config.toml.example` to `config.toml`
3. Fill in your bot token and webhook secret
4. Get your Telegram chat ID (can use [@userinfobot](https://t.me/userinfobot))

## Running

```bash
cargo run
```

## Configuration

| Field | Description |
|-------|-------------|
| `server.host` | Bind address |
| `server.port` | Port to listen on |
| `telegram.bot_token` | Bot token from BotFather |
| `github.webhook_secret` | Secret for webhook signature verification |
| `routing[].repo_pattern` | Repo pattern (`*`, `org/*`, `org/repo`) |
| `routing[].chat_id` | Telegram chat/group ID |
| `routing[].events` | List of events to forward |

## GitHub Webhook Setup

1. Go to repo Settings → Webhooks → Add webhook
2. Payload URL: `https://your-domain/webhook/github`
3. Content type: `application/json`
4. Secret: same as `github.webhook_secret`
5. Select events to send

## Supported Events

- `pull_request.opened`, `pull_request.closed`, `pull_request.reopened`
- `issues.opened`, `issues.closed`, `issues.reopened`
- `push`
- `workflow_run` (GitHub Actions)
- `release`

## Docker

```bash
docker build -t gh-telegram-forwarder .
docker run -v ./config.toml:/app/config.toml gh-telegram-forwarder
```
