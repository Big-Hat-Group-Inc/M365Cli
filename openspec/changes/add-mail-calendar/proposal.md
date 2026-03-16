# Change Proposal: Add Mail and Calendar MVP

## Why

Mail and calendar are core interactive workloads in `spec.md` and define much of the CLI’s day-to-day value. They also exercise the most important delegated Graph behaviors: listing, filtering, reading, sending, calendar ranges, and write operations with least-privilege permissions.

## What Changes

- Define the MVP mail command set for list, search, read, send, and attachment download.
- Define the MVP calendar command set for day or week views plus create, update, and delete of single events.
- Define shared output expectations, filter behavior, and search semantics for these workloads.
- Define permission requirements and read-versus-write behavior for both modules.

## Scope

In scope:

- `mog mail` MVP commands
- `mog calendar` MVP commands
- Attachment download
- Calendar view expansion using `calendarView`

Out of scope:

- Contacts, people, tasks, directory
- Calendar recurrence authoring
- Teams or broader Exchange administration

## Impact

- Delivers the core human-facing CLI scenarios described in the source spec.
- Validates the command ergonomics, output model, and delegated auth model with real Graph workloads.
