# Tasks

## 1. Mail module

- [ ] Implement mail list with headers-only default output.
- [ ] Implement mail search with KQL and result limiting.
- [ ] Implement full message read.
- [ ] Implement send mail with body-file and attachment support.
- [ ] Implement attachment download to a target directory.

## 2. Calendar module

- [ ] Implement today and week views using `calendarView`.
- [ ] Implement single-event create, update, and delete.
- [ ] Implement range and timezone handling for calendar output.

## 3. Shared UX and output

- [ ] Implement default field selection and compact table or plain rendering.
- [ ] Implement JSON output for all mail and calendar commands.
- [ ] Implement permission-aware error messages for read versus write operations.

## 4. Verification

- [ ] Add integration tests against seeded mailboxes and calendars.
- [ ] Add regression tests for plain and JSON output formats.
- [ ] Add tests covering attachment download and event CRUD behavior.
