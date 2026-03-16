# Change Proposal: Add Authentication and Profiles

## Why

`spec.md` makes authentication and profile management a first-class MVP capability. The CLI must support delegated and unattended access patterns, least-privilege consent, secure token storage, and tenant-aware multi-profile switching before workload commands can be used safely.

## What Changes

- Define profile CRUD and profile resolution behavior.
- Define authentication flows for device code, browser PKCE, client credentials, managed identity, and federated identity.
- Define secure token cache behavior and keyring-backed secret storage.
- Define scope bundle mapping, missing-scope remediation, and permission explanation commands.
- Define logout, token invalidation, and consent helper behavior.

## Scope

In scope:

- `mog auth` command set
- Profile store schema and platform paths
- Token cache security model
- Least-privilege scope selection
- Admin consent helper behavior

Out of scope:

- Workload command implementations
- Sovereign cloud validation beyond the preset contract

## Impact

- Enables interactive and automation usage patterns described in the product spec.
- Locks the security and permission model before data access features are introduced.
- Gives later proposals a consistent way to declare and request scopes.
