# Change Proposal: Add Post-MVP Modules

## Why

The MVP delivers mail, calendar, and files. The next tier of workloads — contacts, people, tasks, and directory — rounds out the everyday productivity surface and exercises additional Graph patterns such as search with `ConsistencyLevel: eventual`, To Do task CRUD, and organization directory queries.

## What Changes

- Define the contacts command set for listing, searching, and reading personal contacts.
- Define the people command set for relevance-ranked and searched people.
- Define the tasks (To Do) command set for list and task CRUD including completion.
- Define the directory command set for user and group lookup, search, and group membership.
- Add scope bundle entries and permission mappings for all new commands.

## Scope

In scope:

- `mog contacts` — list, search, read personal contacts
- `mog people` — relevant people and search
- `mog tasks` — To Do list and task CRUD
- `mog directory` — users, groups, group members

Out of scope:

- Org contacts (admin-only Exchange resources)
- Planner tasks (separate Planner API)
- Privileged directory operations (role assignments, admin units)
- Batch operations across modules

## Impact

- Adds four new workspace crates following the established pattern.
- Adds four new CLI subcommand groups with consistent UX.
- Extends scope bundle mappings and permission explanations.
- No changes to existing modules or breaking changes.
