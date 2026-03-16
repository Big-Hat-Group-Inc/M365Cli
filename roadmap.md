# mog Roadmap — Deferred Items

Items deferred from v1 MVP. Organized by priority tier and category.

---

## Tier 1: Near-Term (v1.x / v2.0)

### Post-MVP Modules

These modules are designed into the architecture but not implemented in v1.

| Module | Commands | Required Permissions |
|---|---|---|
| **contacts** | `mog contacts list`, `search`, `read` | `Contacts.Read` |
| **people** | `mog people search`, `relevant` | `People.Read` / `People.Read.All` |
| **tasks** | `mog tasks lists`, `list`, `create`, `update`, `complete`, `delete` | `Tasks.ReadWrite` |
| **directory** | `mog directory users`, `groups`, `search` | `User.ReadBasic.All` / `User.Read.All` |

### JMESPath `--query` Implementation

**Deferred from:** Q17 — Global Flags

The `--query <jmespath>` flag is defined in global flags but **not implemented in v1**. Users should use `--json | jq` for client-side filtering.

**v2 plan:** Integrate a JMESPath library (e.g., `jmespath` Rust crate) to enable `--query` for in-process result filtering without requiring `jq`.

### Additional Distribution Channels

**Deferred from:** Q32 — Packaging

| Channel | Notes |
|---|---|
| **Scoop** | Windows package manager (alternative to winget) |
| **apt/deb** | Debian/Ubuntu native packaging |
| **rpm** | RHEL/Fedora native packaging |
| **Snap** | Consider based on demand |

---

## Tier 2: Medium-Term (v2.x)

### Calendar Recurrence Creation

**Deferred from:** Q13 — Calendar Recurrence

v1 supports **read-only** recurrence (expanded occurrences via `calendarView`). Creating recurring events requires constructing Graph `recurrence` objects with patterns and ranges.

**Proposed DSL:**

```bash
# Weekly on Tuesday and Thursday
mog calendar create --subject "Standup" --start "09:00" --end "09:30" \
  --recurrence "weekly:tue,thu"

# Monthly on the 15th
mog calendar create --subject "Review" --start "14:00" --end "15:00" \
  --recurrence "monthly:15"

# Daily for 10 occurrences
mog calendar create --subject "Sprint" --start "09:00" --end "09:15" \
  --recurrence "daily" --recurrence-count 10

# Yearly on March 1
mog calendar create --subject "Annual Review" --start "10:00" --end "11:00" \
  --recurrence "yearly:mar-1"
```

**Implementation considerations:**
- Parse DSL → Graph `recurrence` object (pattern + range)
- Support `--recurrence-end <date>` for date-bounded series
- Support `--recurrence-count <n>` for count-bounded series
- Handle timezone correctly (`--timezone` flag or profile default)

### Runtime-Loadable Plugins

**Deferred from:** Q3 — Plugin System

v1 uses compiled-in modules only. Runtime plugins would allow third-party module development without rebuilding `mog`.

**Design considerations:**
- Plugin discovery: `~/.mog/plugins/` directory
- Plugin format: shared libraries (`.so`/`.dylib`/`.dll`) or WASM modules
- Security: sandboxing, permission model for plugins
- Versioning: plugin API stability guarantees
- Distribution: plugin registry or manual installation

**WASM option (preferred for security):**
- Plugins compiled to WASM, run in a sandboxed runtime (`wasmtime`)
- Cross-platform without recompilation
- Memory-safe isolation from host process
- Performance overhead acceptable for CLI workloads

### Sovereign Cloud Full Testing

**Deferred from:** Q8 — Sovereign Clouds

v1 includes cloud presets (`gcc`, `gcc-high`, `dod`, `china`) with configurable authority and Graph base URLs, but they are **untested**.

**Validation plan:**
- Obtain test tenants in each sovereign cloud
- Validate auth flows (device code, browser, client credentials) per cloud
- Verify Graph endpoint compatibility (some endpoints behave differently)
- Document any sovereign-cloud-specific limitations
- Test token cache behavior across cloud boundaries

**Known differences to validate:**
- GCC-High/DoD: `login.microsoftonline.us` authority, `graph.microsoft.us` endpoint
- China (21Vianet): `login.chinacloudapi.cn` authority, `microsoftgraph.chinacloudapi.cn` endpoint
- Some Graph features may not be available in all clouds

---

## Tier 3: Long-Term / On-Demand

### Telemetry

**Deferred from:** Q22 — Telemetry Policy

v1 has **no telemetry**. If added in a future version:

- **Opt-in only** — never default-enabled
- Clear disclosure at first run when enabled
- `mog config set telemetry.enabled false` to disable
- Document exactly what is collected and where it goes
- Respect `DO_NOT_TRACK` environment variable
- Consider anonymous aggregate usage stats only (command frequency, error rates)
- Never collect: tenant IDs, user identifiers, email content, file names

### Watch/Sync Commands

- Delta-query-based `--since-last-sync` for incremental mail/calendar sync
- Local state tracking for sync cursors
- Webhook integration for real-time notifications (requires user infrastructure)

### Advanced Output

- CSV export with configurable delimiters
- Custom output templates
- Markdown table output

---

## Pre-Implementation Research Tasks

### Graph API Surface for Module Interface Contract

**From:** Q4 — Module Interface Contract

The workload module trait design in the spec is illustrative (TypeScript example translated to Rust). Before implementation, research:

1. **Required Graph API surface** for each MVP module (exact endpoints, query parameters, response shapes)
2. **Shared patterns** across modules (pagination, `$select`, `$filter`, `$search`, `$orderby`)
3. **Optimal Rust trait design** — should modules return typed structs or generic `serde_json::Value`?
4. **Error propagation** — how Graph error codes map to module-level errors
5. **Scope declaration** — static metadata vs. runtime introspection

**Output:** Finalized Rust trait definitions for `WorkloadModule`, `GraphClient`, and shared types.
