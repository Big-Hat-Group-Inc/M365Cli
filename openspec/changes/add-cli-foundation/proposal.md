# Change Proposal: Add CLI Foundation

## Why

`spec.md` defines `mog` as a single cross-platform Rust CLI with strict stdout/stderr behavior, stable automation modes, shared configuration, exit code conventions, retry-aware Graph transport, and packaging expectations. None of that structure exists in the repository yet.

Implementing workload commands before the CLI substrate would create avoidable churn in command shape, configuration handling, error modeling, and output contracts.

## What Changes

- Create the Rust workspace and internal crate boundaries for `mog-cli`, `mog-core`, and `mog-graph`.
- Define the global CLI contract for profile selection, output modes, non-interactive behavior, color handling, and exit codes.
- Define the shared Graph request pipeline contract for retries, pagination, correlation headers, deprecation warnings, and throttling behavior.
- Define persistent global configuration and platform-specific config locations.
- Define packaging, shell completion, and baseline test expectations for the foundation layer.

## Scope

In scope:

- Workspace structure
- Global flags and command conventions
- Output and error rendering contracts
- Shared Graph client behavior
- Config file layout
- Test and packaging requirements for the platform layer

Out of scope:

- Authentication flows
- Mail, calendar, files, or raw Graph feature implementation
- Post-MVP modules from `roadmap.md`

## Impact

- Establishes the baseline contract all later module proposals must follow.
- Reduces rework by locking shared CLI behavior before workload implementation.
- Enables parallel development of workload modules against a stable platform contract.
