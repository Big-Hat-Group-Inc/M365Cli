# OpenSpec

This directory contains the normative specifications and change history for `mog`.

## Applied Specs

The `specs/` directory contains the current normative requirements:

| Spec | Source Proposal | Status |
|---|---|---|
| `specs/cli-foundation/spec.md` | `add-cli-foundation` | Applied |
| `specs/auth/spec.md` | `add-auth-profiles` | Applied |
| `specs/mail/spec.md` | `add-mail-calendar` | Applied |
| `specs/calendar/spec.md` | `add-mail-calendar` | Applied |
| `specs/files/spec.md` | `add-files-graph` | Applied |
| `specs/graph/spec.md` | `add-files-graph` | Applied |
| `specs/contacts/spec.md` | `add-post-mvp-modules` | Applied |
| `specs/people/spec.md` | `add-post-mvp-modules` | Applied |
| `specs/tasks/spec.md` | `add-post-mvp-modules` | Applied |
| `specs/directory/spec.md` | `add-post-mvp-modules` | Applied |
| `specs/security/spec.md` | `add-security-hardening` | Applied |
| `specs/developer-experience/spec.md` | `add-developer-experience` | Applied |

## Change History

The `changes/` directory preserves the original proposals that produced the specs above.

| Proposal | Description | Status |
|---|---|---|
| `add-cli-foundation` | Workspace structure, global CLI behavior, Graph transport, config | Applied |
| `add-auth-profiles` | Authentication flows, profile CRUD, token cache, scope bundles | Applied |
| `add-mail-calendar` | Mail list/search/read/send, calendar views and event CRUD | Applied |
| `add-files-graph` | Files search/download/upload, raw Graph escape hatch | Applied |
| `add-post-mvp-modules` | Contacts, people, tasks (To Do), directory (users/groups) | Applied |
| `add-security-hardening` | Security fixes, dependency cleanup, async I/O, structured logging | Applied |
| `add-developer-experience` | Workspace linting, crate docs, cargo fmt, CI/CD workflow | Applied |

Each change contains:

- `proposal.md`: problem statement, scope, and impact
- `tasks.md`: implementation checklist (all complete)
- `specs/*/spec.md`: normative requirements and scenarios
