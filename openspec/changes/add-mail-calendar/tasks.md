# Tasks

## 1. Mail module

- [x] Implement mail list with headers-only default output.
- [x] Implement mail search with KQL and result limiting.
- [x] Implement full message read.
- [x] Implement send mail with body-file and attachment support.
- [x] Implement attachment download to a target directory.

## 2. Calendar module

- [x] Implement today and week views using `calendarView`.
- [x] Implement single-event create, update, and delete.
- [x] Implement range and timezone handling for calendar output.

## 3. Shared UX and output

- [x] Implement default field selection and compact table or plain rendering.
- [x] Implement JSON output for all mail and calendar commands.
- [x] Implement permission-aware error messages for read versus write operations.

## 4. Verification

- [x] Add integration tests against seeded mailboxes and calendars.
- [x] Add regression tests for plain and JSON output formats.
- [x] Add tests covering attachment download and event CRUD behavior.
