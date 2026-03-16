# questions.md — Implementation Blockers & Clarifications

Items that need resolution before implementation can begin. Grouped by category. Each includes what the spec says (or doesn't), why it matters, and a suggested default where obvious.

---

## Architecture & Language

### Q1: What language/runtime?

**Spec says:** Explicitly language-agnostic ("assumes no OS/language constraint"). Mentions Node.js, .NET, and Go as possibilities.

**Why it matters:** Everything downstream depends on this — dependency management, packaging, Graph SDK availability, MSAL library choice, binary distribution. This is the single highest-impact decision.

**Suggested default:** Go or Rust for true single-binary distribution (matches the "gog-like" goal). Node.js is viable but requires bundling or a runtime dependency (like CLI for M365). .NET gives native Graph SDK support but AOT compilation is still maturing for complex apps. - Use RUST

### Q2: Monorepo or separate packages per workload module?

**Spec says:** Describes "pluggable workload modules" but doesn't define the repo structure.

**Why it matters:** Affects CI/CD, versioning, contributor workflow, and whether modules can be developed/released independently.

**Suggested default:** Monorepo with internal packages. Workload modules are tightly coupled to the Graph client layer and auth — separate repos add coordination overhead with no real benefit at this scale. - agreed

### Q3: Plugin system — compiled-in or runtime-loadable?

**Spec says:** "Pluggable workload modules" but no definition of the plugin interface or loading mechanism.

**Why it matters:** Runtime plugins add complexity (discovery, versioning, security). Compiled-in modules are simpler but require rebuilding for new workloads.

**Suggested default:** Compiled-in modules with a clean internal interface. Runtime plugins are a v2+ concern. - make a roadmap.md note of this.

### Q4: What is the module interface contract?

**Spec says:** Nothing. The architecture diagram shows modules connecting to the Graph client layer but doesn't define the API surface. use perplexity to perform deep research on the API surface needed.

**Why it matters:** Every workload module developer needs to know: what do I implement? How do I register commands? How do I declare required permissions?

**Needs answer before implementation.**

---

## Authentication & Identity

### Q5: App registration model — shared or BYO?

**Spec says:** Nothing explicit. Implies an app registration exists but doesn't say who creates it. 

**Why it matters:** A shared multi-tenant app registration (like CLI for M365 ships) means users can `mog auth login` immediately. BYO means every user must create their own app reg first — massive friction for adoption.

**Suggested default:** Ship with a pre-registered multi-tenant app (public client, no secret). Support `--client-id` override for orgs that want their own. - have a powershell script for making the application registration and installing all the prerequisites.

### Q6: What is the default client ID?

**Spec says:** Nothing.

**Why it matters:** The app registration must exist before the first user can authenticate. Needs to be created, configured with redirect URIs (localhost for browser flow, `urn:ietf:wg:oauth:2.0:oob` for device code), and have the correct API permissions pre-configured.

**Needs answer. Must be created and published before v1.** Use best practice for redirect.

### Q7: Certificate auth — PFX, PEM, or both?

**Spec says:** Mentions `--certificate-file <path> [--thumbprint ...]` but doesn't specify supported formats.

**Why it matters:** PFX is Windows-native. PEM is Linux/macOS-native. Supporting both adds complexity but is expected.

**Suggested default:** Both. PEM is primary (cross-platform), PFX supported for Windows environments. Certificate for windows but file support is needed for non-windows.

### Q8: Sovereign cloud support?

**Spec says:** Nothing about GCC, GCC-High, DoD, or China (21Vianet) clouds.

**Why it matters:** These clouds use different authority endpoints (`login.microsoftonline.us`, `login.chinacloudapi.cn`) and Graph endpoints (`graph.microsoft.us`, `microsoftgraph.chinacloudapi.cn`). If not designed in early, retrofitting is painful.

**Suggested default:** Design the auth layer to accept configurable authority and Graph base URLs from the start. Ship with presets for `commercial`, `gcc`, `gcc-high`, `dod`, `china`. Don't block v1 on testing all of them, but don't hardcode `login.microsoftonline.com` either. - Add these to roadmap.

### Q9: Token cache location and encryption?

**Spec says:** "Secure token cache persistence" and mentions MSAL Node extensions, but no concrete location or encryption scheme.

**Why it matters:** Needs a defined file path per platform, encryption-at-rest strategy, and multi-process concurrency handling.

**Suggested default:**

- Windows: `%LOCALAPPDATA%\mog\token_cache.bin` (DPAPI encrypted)
- macOS: Keychain
- Linux: `~/.mog/token_cache.bin` (libsecret or encrypted with user passphrase fallback)

---

## Data Handling

### Q10: Mail body — full or summary by default?

**Spec says:** Mentions `--include-body` as an explicit opt-in (mirroring gog), but doesn't define the default behavior for `mog mail list`.

**Why it matters:** Full bodies can be huge (HTML emails with inline images). Returning them by default bloats JSON output and slows API calls (Graph returns body by default unless you use `$select`).

**Suggested default:** `mog mail list` returns headers only (from, to, subject, date, preview). `mog mail read <id>` returns full body. `--include-body` on list commands opts in.

### Q11: Attachment handling thresholds?

**Spec says:** Nothing about inline vs download-to-file behavior for attachments.

**Why it matters:** A 50MB attachment shouldn't be base64-encoded into JSON stdout. Need a size threshold and a download path.

**Suggested default:** Attachments listed as metadata by default. `mog mail attachments download <messageId>` saves to files. Never inline binary content in JSON output.

### Q12: File upload chunk size?

**Spec says:** Mentions resumable upload sessions but no chunk size.

**Why it matters:** Graph recommends 5-10MB chunks for upload sessions. Too small = too many requests. Too large = memory pressure and timeout risk.

**Suggested default:** 5MB default, configurable via `--chunk-size`.

### Q13: Calendar recurrence — read-only or read-write?

**Spec says:** "Create/update events" and "handle common recurrence patterns where supported" — vague.

**Why it matters:** Creating recurrence rules via Graph requires constructing `recurrence` objects with patterns and ranges. This is non-trivial UX (how does a user express "every Tuesday and Thursday" on a CLI?).

**Suggested default:** MVP: read-only recurrence (display expanded occurrences via calendarView). v2: support creating recurring events with a limited DSL (e.g., `--recurrence "weekly:tue,thu"`).

---

## Output & Formatting

### Q15: Exit code conventions?

**Spec says:** Nothing.

**Why it matters:** Scripts depend on exit codes. Without conventions, automation is unreliable.

**Suggested default:**

- `0` — success
- `1` — general error
- `2` — authentication failure / token expired
- `3` — permission denied (403)
- `4` — resource not found (404)
- `5` — throttled (429, after retries exhausted)

### Q16: `NO_COLOR` and ANSI control?

**Spec says:** Nothing about color output or the `NO_COLOR` standard.

**Why it matters:** CI/CD environments and piped output should not include ANSI escape codes.

**Suggested default:** Support `NO_COLOR` env var (standard). Auto-detect TTY — no color when piped. `--color always|never|auto` flag.

### Q17: JMESPath `--query` — hard requirement?

**Spec says:** Lists it as an optional richer format alongside `--output`.

**Why it matters:** JMESPath adds a dependency and documentation burden. It's powerful but rarely used by most CLI users.

**Suggested default:** Nice-to-have for v1. Implement `--json` + pipe to `jq` as the primary path. Add `--query` in v2 if demand warrants.

### Q18: Table rendering approach?

**Spec says:** Nothing about how `--plain` or default table output is rendered.

**Why it matters:** Column widths, truncation, Unicode handling, terminal width detection — all affect usability.

**Needs answer. Pick a library or define the format spec (fixed-width columns? tab-separated?).** Format tables cleanly.

---

## Permissions & Governance

### Q19: Scope bundle definitions — hardcoded or configurable?

**Spec says:** Shows example bundles (`mail:read` → `Mail.Read`) but doesn't say where these mappings live.

**Why it matters:** Hardcoded bundles are simpler but can't adapt to tenant-specific permission models. Configurable bundles add complexity.

**Suggested default:** Hardcoded in the CLI with a version-bumped mapping file. Document the full mapping. Don't make it user-configurable — that invites misconfiguration.

### Q20: What happens when a tenant blocks required permissions entirely?

**Spec says:** Mentions admin consent workflows but doesn't define CLI behavior when consent is permanently denied.

**Why it matters:** The CLI shouldn't retry forever or give cryptic errors.

**Suggested default:** Clear error message explaining which permission is missing, why it's needed, and what admin action is required. Link to admin consent URL. Exit with code 3.

---

## Operational

### Q22: Telemetry?

**Spec says:** Nothing.

**Why it matters:** Privacy-sensitive decision. Some orgs block CLIs that phone home.

**Suggested default:** No telemetry in v1. If added later, opt-in only with clear disclosure.

### Q23: Config file format and location?

**Spec says:** Mentions `profiles.json` once but doesn't define the schema, format, or platform-specific paths.

**Why it matters:** Fundamental to the profile/auth system. Needs a defined schema before auth implementation begins.

**Suggested default:** JSON config at:
- Windows: `%LOCALAPPDATA%\mog\config.json`
- macOS/Linux: `~/.config/mog/config.json`

Define the schema (profiles array, default profile, global settings).

### Q24: How does `mog graph call` work?

**Spec says:** "Raw Graph call escape hatch" — no further detail.

**Why it matters:** This is the power-user fallback. Needs a clear interface.

**Suggested default:** `mog graph call GET /me/messages --top 5 --header "Prefer: outlook.body-content-type=text"` — verb + path + optional query params + headers + body (from stdin or `--body-file`).

### Q25: Signal handling during long operations?

**Spec says:** Nothing.

**Why it matters:** SIGINT during a large upload session should clean up gracefully (cancel the upload session on the server). SIGINT during a batch should not leave partial state.

**Suggested default:** Trap SIGINT/SIGTERM. Cancel in-flight upload sessions. For batch operations, complete the current batch request but don't send more. Print what succeeded and what didn't.

### Q26: Offline behavior?

**Spec says:** Nothing about what happens when Graph is unreachable.

**Why it matters:** Network failures are common. The CLI should fail fast with a clear error, not hang.

**Suggested default:** Connection timeout of 10s. Clear error: "Cannot reach Microsoft Graph. Check network connectivity." No offline caching or queued operations in v1.

---

## Error Handling

### Q27: Which Graph error codes get special handling beyond 429?

**Spec says:** Only mentions 429 (throttling) explicitly.

**Why it matters:** 401 (token expired), 403 (forbidden), 404 (not found), 409 (conflict), 503 (service unavailable) all need distinct CLI behavior and user-facing messages.

**Suggested default:** Define handling for at minimum: 400, 401, 403, 404, 409, 429, 500, 502, 503, 504. Map each to an exit code and a human-readable message template.

### Q28: Pagination strategy?

**Spec says:** Mentions paging exists but doesn't define CLI behavior.

**Why it matters:** Some Graph endpoints return thousands of results across many pages. `mog mail list` with no `--top` flag could auto-paginate through 10,000 messages.

**Suggested default:** Default `--top 25`. Auto-paginate up to `--max` (default 100). `--all` flag to paginate through everything (with a warning). Stream results as they arrive for `--json`.

### Q29: Batch request partial failure handling?

**Spec says:** Nothing.

**Why it matters:** Graph batch requests can partially succeed (some items 200, others 429 or 404). The CLI needs to report partial results and retry failed items.

**Suggested default:** Report all results. Retry 429s within the batch. Surface non-retryable failures with per-item error details. Exit code 1 if any item failed.

---

## Testing & CI

### Q30: Test tenant provisioning?

**Spec says:** "Use a dedicated test tenant and test users/mailboxes" — no detail on who creates this or how.

**Why it matters:** Integration tests need real Graph endpoints, real mailboxes, real files. This requires a tenant with test data fixtures.

**Suggested default:** Use the [Microsoft 365 Developer Program](https://developer.microsoft.com/en-us/microsoft-365/dev-program) for a free E5 dev tenant. Document required test data fixtures (test mailbox, calendar events, OneDrive files, To Do lists). Automate fixture creation with a setup script.

### Q31: Mock strategy for unit tests?

**Spec says:** Nothing.

**Why it matters:** Unit tests shouldn't hit Graph. Need a mocking strategy that's maintainable.

**Suggested default:** HTTP recording/replay (like `nock` for Node, `httpmock` for Rust, `httptest` for Go). Record responses from the test tenant, replay in CI.

---

## Packaging & Distribution

### Q32: Package manager targets?

**Spec says:** "Homebrew, build from source" mentioned as gog references. No concrete list.

**Why it matters:** Determines CI/CD pipeline complexity and release process.

**Suggested default:** v1: GitHub Releases (binaries) + Homebrew tap + winget. v2: npm (if Node), Scoop, apt/deb.

### Q33: Shell completions?

**Spec says:** Nothing.

**Why it matters:** Tab completion is a major usability feature for CLIs. Needs to be designed into the command registration system.

**Suggested default:** Ship completions for bash, zsh, fish, and PowerShell. Generate from command definitions (most CLI frameworks support this natively).

### Q34: Minimum OS/runtime versions?

**Spec says:** Nothing concrete beyond "cross-platform."

**Why it matters:** Affects CI matrix and which APIs/libraries are available.

**Suggested default:** Windows 10+, macOS 12+, Ubuntu 20.04+ / RHEL 8+. If Node: Node 20 LTS+. If Go/Rust: ship static binaries.

---

## MVP Scope

### Q35: What's in v1?

**Spec says:** Describes the full vision but doesn't prioritize. All workloads are presented equally.

**Why it matters:** Trying to ship mail + calendar + files + contacts + people + tasks + directory + graph in v1 is a recipe for shipping nothing.

**Suggested default MVP:**
1. `auth` (device code + browser + client credentials + profiles)
2. `mail` (list, search, read, send, attachments)
3. `calendar` (today, week, calendarView, create event)
4. `files` (list, search, download, upload)
5. `graph` (raw escape hatch)
6. `config` + `version`

Everything else is v2+.
