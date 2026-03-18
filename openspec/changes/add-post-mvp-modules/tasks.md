# Tasks

## 1. Contacts module

- [x] Create `mog-contacts` crate with Cargo.toml and workspace registration.
- [x] Implement `list_contacts` with default field selection and `$top`/`$filter`.
- [x] Implement `search_contacts` with `$search` query parameter.
- [x] Implement `read_contact` for single contact retrieval.
- [x] Add CLI subcommands and dispatch wiring.

## 2. People module

- [x] Create `mog-people` crate with Cargo.toml and workspace registration.
- [x] Implement `relevant_people` for ranked people list.
- [x] Implement `search_people` with `$search` query parameter.
- [x] Add CLI subcommands and dispatch wiring.

## 3. Tasks (To Do) module

- [x] Create `mog-tasks` crate with Cargo.toml and workspace registration.
- [x] Implement `list_task_lists` for To Do list enumeration.
- [x] Implement `list_tasks` with optional status filtering.
- [x] Implement `create_task` with title, body, and due date.
- [x] Implement `update_task` for field patching.
- [x] Implement `complete_task` as a convenience wrapper.
- [x] Implement `delete_task` for task removal.
- [x] Add CLI subcommands and dispatch wiring.

## 4. Directory module

- [x] Create `mog-directory` crate with Cargo.toml and workspace registration.
- [x] Implement `list_users` with `$top`/`$filter`.
- [x] Implement `search_users` with `ConsistencyLevel: eventual` header.
- [x] Implement `read_user` for single user lookup.
- [x] Implement `list_groups` with `$top`/`$filter`.
- [x] Implement `search_groups` with `ConsistencyLevel: eventual` header.
- [x] Implement `read_group` for single group lookup.
- [x] Implement `list_group_members` for group membership.
- [x] Add CLI subcommands and dispatch wiring.

## 5. Shared integration

- [x] Add scope bundle entries for contacts, people, tasks, directory.
- [x] Add `command_scopes()` entries for all new commands.
- [x] Add `explain_permissions()` entries for new permission strings.
- [x] Update `openspec/README.md` with new proposal entry.

## 6. Verification

- [x] Add unit tests for format helpers and field selection in each new crate.
- [x] Add integration tests for command parsing in CLI.
- [x] Verify `cargo test` passes for all crates.
