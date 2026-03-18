# mog — Microsoft 365 CLI

A cross-platform CLI for Microsoft 365 via Microsoft Graph. Built in Rust as a single static binary with gog-like ergonomics: multi-account profiles, JSON-first output, least-privilege auth, and stdout/stderr discipline.

## Features

- **Mail** — list, search (KQL), read, send with attachments
- **Calendar** — day/week views, create/update/delete events
- **Files** — search, download, upload (resumable), format conversion
- **Contacts** — list, search, read personal contacts
- **People** — relevance-ranked people lookup
- **Tasks** — Microsoft To Do lists and tasks CRUD
- **Directory** — user and group lookup
- **Raw Graph** — escape hatch for any Graph API call
- **Multi-account** — named profiles with per-profile tenant, auth strategy, and scopes
- **Auth flows** — device code, browser PKCE, client credentials (cert/secret), managed identity, federated identity

## Installation

### From source (requires Rust toolchain)

```bash
git clone https://github.com/bighatgroup/mog.git
cd mog
cargo build --release
# Binary is at target/release/mog (or target/release/mog.exe on Windows)
```

Add to your PATH:

```bash
# Linux/macOS
cp target/release/mog ~/.local/bin/

# Windows (PowerShell)
Copy-Item target\release\mog.exe "$env:USERPROFILE\.local\bin\"
```

### From cargo

```bash
cargo install --path crates/mog-cli
```

### Planned distribution channels (v1 release)

| Channel | Command |
|---------|---------|
| Homebrew | `brew install mog-cli/tap/mog` |
| winget | `winget install mog` |
| cargo | `cargo install mog` |
| GitHub Releases | Pre-built binaries + SHA-256 checksums |

### Shell completions

```bash
mog completions bash > ~/.local/share/bash-completion/completions/mog
mog completions zsh > ~/.zfunc/_mog
mog completions fish > ~/.config/fish/completions/mog.fish
mog completions powershell > "$PROFILE\..\Completions\mog.ps1"
```

## Quick Start

### 1. Log in

```bash
# Interactive device code flow (default)
mog auth login --profile work --tenant contoso.onmicrosoft.com \
  --services mail,calendar --readonly

# Browser flow (when device code is blocked by Conditional Access)
mog auth login --profile work --tenant contoso.onmicrosoft.com \
  --strategy browser --services mail,calendar
```

### 2. Check status

```bash
mog auth status
```

### 3. Use it

```bash
# List unread mail from the last 7 days
mog mail list --unread --since 7d --json

# Search mail with KQL
mog mail search --kql 'from:alice AND hasAttachments:true' --top 10

# Read a message
mog mail read <message-id>

# Today's calendar
mog calendar today

# Search OneDrive
mog files search -q "quarterly report" --json

# Raw Graph API call
mog graph call GET /me/memberOf --json
```

## Usage

### Global Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--profile <name>` | Named profile to use | `MOG_PROFILE` env or config default |
| `--json` | JSON output | off |
| `--plain` | Stable plain-text output | off |
| `--output <format>` | Output format: json, text, csv, table | table (TTY) / json (pipe) |
| `--no-input` | Never prompt; fail instead | auto when no TTY |
| `--verbose` | Verbose logging to stderr | off |
| `--debug` | Full debug output including raw HTTP | off |
| `--trace` | JSON trace event per Graph call | off |
| `--api-version <v>` | Graph API version: v1.0, beta | v1.0 |
| `--top <n>` | Server-side page size | 25 |
| `--all` | Auto-paginate all results | off |
| `--force` | Skip destructive confirmations | off |
| `--client-id <id>` | Override app registration | built-in default |
| `--tenant <id>` | Override tenant | from profile |
| `--color <mode>` | Color: auto, always, never | auto |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `MOG_PROFILE` | Default profile name |
| `MOG_CONFIG_DIR` | Override config directory |
| `NO_COLOR` | Disable color output |

### Command Reference

#### Auth

```bash
mog auth login [flags]           # Log in (device code, browser, app, MI, federated)
mog auth logout                  # Clear cached tokens
mog auth status                  # Show current auth status
mog auth profile list            # List profiles
mog auth profile create <name>   # Create a profile
mog auth profile update <name>   # Update a profile
mog auth profile delete <name>   # Delete a profile
mog auth profile set-default <n> # Set default profile
mog auth explain-permissions <cmd>  # Show required permissions for a command
mog auth admin-consent-url       # Generate admin consent URL
```

#### Mail

```bash
mog mail list [--unread] [--since 7d] [--include-body]
mog mail search --kql '<KQL query>'
mog mail read <message-id> [--body-type text]
mog mail send --to <email> --subject <subj> [--body-file f] [--attach f]
mog mail attachments list <message-id>
mog mail attachments download <message-id> [--out-dir dir]
```

#### Calendar

```bash
mog calendar today
mog calendar week
mog calendar list --start 2025-01-01 --end 2025-01-31
mog calendar create --subject "Meeting" --start "2025-01-20T09:00" --end "2025-01-20T10:00"
mog calendar update <event-id> --subject "New Subject"
mog calendar delete <event-id>
```

#### Files

```bash
mog files search -q "query"
mog files download --item-id <id> --out ./file.bin
mog files export --item-id <id> --format pdf --out ./file.pdf
mog files upload --dest "/path/file" --file ./local [--resumable] [--chunk-size 10485760]
```

#### Contacts, People, Tasks, Directory

```bash
mog contacts list [--filter <expr>]
mog contacts search --query "name"
mog people relevant
mog people search --query "name"
mog tasks lists
mog tasks list --list-id <id>
mog tasks create --list-id <id> --title "Task"
mog directory users list
mog directory users search --query "name"
mog directory groups list
mog directory groups members --group-id <id>
```

#### Raw Graph

```bash
mog graph call GET /me/messages --top 5 --json
mog graph call POST /me/sendMail --body-file payload.json
echo '{"key":"value"}' | mog graph call PATCH /me --body-stdin
```

#### Config

```bash
mog config show
mog config set output.defaultFormat json
```

### CI/CD Usage

```yaml
# GitHub Actions with workload identity federation (recommended)
- run: mog auth login --auth-type federated --tenant $TENANT_ID --client-id $CLIENT_ID --federated-token-file $TOKEN_FILE
- run: mog mail list --unread --since 1d --json

# Azure with managed identity
- run: mog auth login --auth-type managed-identity
- run: mog files search -q "report" --json
```

### Service Principal Authentication

```bash
# Certificate-based (preferred)
mog auth login --auth-type app --tenant <id> --client-id <id> --certificate-file ./cert.pem

# Client secret via stdin (discouraged)
echo "$SECRET" | mog auth login --auth-type app --tenant <id> --client-id <id> --client-secret-stdin
```

## Configuration

Config files are stored at platform-specific locations:

| Platform | Path |
|----------|------|
| Windows | `%LOCALAPPDATA%\mog\` |
| macOS | `~/.config/mog/` |
| Linux | `~/.config/mog/` |

Override with `MOG_CONFIG_DIR` environment variable.

Files:
- `profiles.json` — named auth profiles
- `config.json` — global settings (output format, retry, pagination)
- `token_cache.json` — cached auth tokens (restricted permissions)

## Troubleshooting

### Authentication Issues

**"Device code flow blocked" or AADSTS50199/AADSTS7000218**
Your tenant's Conditional Access policy blocks device code flow. Use browser flow instead:
```bash
mog auth login --strategy browser --services mail,calendar
```

**"Consent required" or AADSTS65001**
Your tenant requires admin consent. Generate the admin consent URL and share with your admin:
```bash
mog auth admin-consent-url
```

**"Insufficient permissions" (exit code 3)**
The current token doesn't have the required scopes. Check what's needed:
```bash
mog auth explain-permissions mail send
```
Then re-login with the right scopes:
```bash
mog auth login --services mail --add-scopes Mail.Send
```

**Token expired / "Authentication failure" (exit code 2)**
Your cached token has expired and silent refresh failed. Log in again:
```bash
mog auth login
```

### Network Issues

**"Cannot reach Microsoft Graph" (exit code 6)**
Check your internet connection. The CLI retries with exponential backoff (configurable via `config.json`). If behind a proxy, ensure HTTPS traffic to `graph.microsoft.com` is allowed.

**Rate limited (exit code 5)**
Graph API throttled requests. The CLI automatically retries with `Retry-After` header respect. For batch operations, reduce concurrency or add delays between runs.

### Output Issues

**No output / empty table**
Check that data exists for the query. Use `--json` for raw output to see exactly what Graph returned:
```bash
mog mail list --json
```

**Garbled output in pipes**
Use `--plain` or `--json` for stable machine-readable output. Table formatting uses Unicode box-drawing characters that may not render in all terminals:
```bash
mog mail list --plain | head -20
mog calendar today --json | jq '.[] | .subject'
```

### Build Issues

**Compilation errors with keyring crate on Linux**
Install the required system libraries:
```bash
# Debian/Ubuntu
sudo apt-get install libdbus-1-dev pkg-config

# Fedora/RHEL
sudo dnf install dbus-devel pkg-config
```

**TLS errors**
The CLI uses `rustls` (vendored TLS) and should not need OpenSSL. If you see TLS errors, ensure your system's CA certificates are up to date.

### Getting Help

```bash
mog --help              # Top-level help
mog mail --help         # Module help
mog mail send --help    # Command help
mog auth explain-permissions <command>  # Required permissions
```

## License

MIT
