# Contacts, calendar, journal, and other item outputs

The current PSTD product is intentionally focused on email extraction, but readpst already emits several non-mail item families. These are parity gaps, not downstream features to postpone indefinitely.

## Contacts and vCard

With `-cv`, readpst emits RFC 2426-style vCards. The upstream writer covers a broad contact model, including:

- full name, surname, given/middle names, prefix, suffix, nickname;
- up to three email addresses and address descriptions/transport types;
- home, business, and other postal addresses;
- home/business/other phone numbers, fax, mobile, pager, radio, telex, ISDN, car, and assistant phone;
- company, department, job title, profession, manager, assistant, spouse, hobbies, gender, account, location, and home pages;
- birthday and wedding anniversary;
- contact body/comment as notes;
- categories from extended `Keywords` fields;
- RFC 2426 escaping and `VERSION:3.0`.

With `-cl`, it emits a simple `fullname <address>` line. PSTD currently has no typed contact record or contact adapter.

### Required PSTD model

```text
ContactRecord
  source identity and folder
  display/name components
  repeated email endpoints with address type
  home/business/other structured addresses
  repeated phones and web addresses
  organization and role fields
  birthdays/anniversaries
  notes and categories
  extraction status for each field group
```

The original MAPI property and raw value must remain available when a vCard projection cannot represent it.

## Appointments and iCalendar

With appointment output selected, readpst emits a `VCALENDAR`/`VEVENT` component. The writer includes, where available:

- UID derived from source identity;
- creation/modified timestamps and DTSTART/DTEND;
- summary, description, and location;
- timezone string and all-day flag;
- show-as/free-busy state;
- Outlook labels/categories;
- recurrence rule and recurrence start/end;
- reminders as `VALARM` with bounded trigger values;
- categories from the item’s `Keywords` fields.

Recurrence types include daily, weekly, monthly, and yearly forms, with interval, weekdays, day-of-month, month, positional occurrence, count, and termination-date semantics. PSTD currently has no appointment record, recurrence decoder, or iCalendar writer.

The implementation must preserve raw recurrence bytes and any exception/deleted-occurrence evidence before projecting to an RFC 5545 form. A recurrence that cannot be represented exactly must be marked partial rather than silently flattened to one event.

## Journals and vJournal

readpst emits:

```text
BEGIN:VJOURNAL
DTSTAMP / CREATED / LAST-MOD
SUMMARY
DESCRIPTION
DTSTART
END:VJOURNAL
```

PSTD needs a typed journal record for start/end timestamps, type, description, subject, body, create/modify times, categories, and raw properties. It must offer deterministic vJournal output and preserve source bytes when a field is absent.

## Tasks, sticky notes, and other classes

The current readpst process path classifies sticky notes, tasks, and other classes but does not emit dedicated output records. Parity therefore has two levels:

1. match readpst by identifying and explicitly reporting the skipped class;
2. provide stronger typed preservation when PSTD can do so without changing the ordinary email semantics.

At no point may these objects be silently counted as successfully extracted messages.

## Distribution lists

Distribution lists and contacts appear in the pinned java-libpst fixture and are useful for broad item coverage. They are not a separate readpst output mode, so they should be modelled as a contact subtype or explicit item class, not confused with ordinary recipient rows. The richer LDAP/LDIF behaviour belongs to a separate future parity register for `pst2ldif`.

## Mixed folders

The `-t` filter and the upstream change history explicitly support folders containing mixed email, calendar, contact, and journal items. Fixtures must prove that:

- each typed output goes to the correct stream;
- one unsupported class does not suppress sibling items;
- folder totals remain reconciled;
- output filters do not alter canonical discovery counts;
- duplicate display names remain distinct by source identity.

## Planned implementation — `RP-08`

### Readpst logic reviewed

`write_vcard` converts contact fields to UTF-8, emits RFC 2426 `VERSION:3.0`, names, three emails, birthday, home/business/other addresses, labels, phone variants, organization/role, assistant, notes, and categories. `-cv` selects that writer; `-cl` emits a simple name/address list. `write_journal` emits vJournal fields and uses current time as a DTSTAMP fallback. `write_appointment` emits a VCALENDAR/VEVENT with UID, timestamps, summary, description, location, status/transparency, labels/categories, recurrence, and a bounded VALARM. `pst_convert_recurrence` decodes daily/weekly/monthly/yearly raw recurrence bytes but is intentionally limited. `process` sends schedule emails through the email writer and ordinary appointments/journals/contacts through typed writers; tasks/sticky notes/unknowns are not fully emitted.

### Planned PSTD records and serializers

Add typed records in `src/output/metadata.rs` or a dedicated `src/output/items.rs`:

```text
ContactRecord      -> VCardProfile | ContactListProfile
AppointmentRecord  -> ICalendarProfile
JournalRecord      -> VJournalProfile
ScheduleRecord     -> email MIME + linked VEVENT
ReportRecord       -> multipart/report MIME
UnsupportedItem    -> typed metadata + explicit status
```

Each field should be a `FieldEvidence<T>` with raw property reference, decoded value, and representability status. Keep recurrence bytes, exception/deleted-occurrence data, timezone evidence, and categories as raw/typed parallel fields. Use standards-aware serializers rather than string concatenation; output-specific lossy projections must be reported.

### Implementation flow

1. Classify contacts, appointments, journals, schedule emails, reports, tasks, sticky notes, and other classes through `RP-03`.
2. Project all known MAPI fields into typed records, retaining repeated values and unknown properties. Normalize contact email/address/phone types without collapsing native address types.
3. Implement RFC 2426-compatible vCard output and `-cl` list output from the same `ContactRecord`; escape delimiters, newlines, backslashes, and non-ASCII values deterministically.
4. Decode recurrence into a normalized rule with raw bytes and an exact/partial flag. Preserve exceptions and deleted occurrences before emitting RFC 5545 `RRULE`/`EXDATE` data.
5. Serialize appointment/timezone/alarm/status/transparency/categories to iCalendar. Use source UID where validated; otherwise use a stable synthetic UID marked as such.
6. Serialize journals to vJournal with source timestamps. If a DTSTAMP fallback is required for compatibility, mark it synthetic and keep the canonical record unchanged.
7. Link schedule email and calendar component through the embedded/special graph. Do not count one as a replacement for the other.
8. Emit unsupported readpst classes as `skipped_unsupported_type` only when no equivalent readpst output exists; retain stronger typed metadata when available.

### Improvements over readpst

- Preserve the full typed contact and calendar model even when vCard/iCalendar cannot represent a property.
- Retain raw recurrence and exception evidence; never flatten an unrepresentable recurrence into one event without a partial status.
- Use stable source UIDs and synthetic markers rather than current-time or display-name identity.
- Bound and validate alarm/recurrence values while preserving out-of-range raw properties.
- Distinguish an output skip from a parser failure and from an item class that readpst itself does not emit.
- Use standards-compliant line folding/escaping and round-trip parsers rather than unchecked `fprintf` output.

### Issue-ready acceptance

`RP-08A` is contact projection/vCard/list, `RP-08B` recurrence/timezone, `RP-08C` appointment/iCalendar, `RP-08D` journal/vJournal, and `RP-08E` unsupported/task/sticky/other classification. Fixtures must cover every contact field group, distribution list, Unicode/ANSI names, repeated phones/emails, recurring daily/weekly/monthly/yearly events, exceptions, alarms, all-day/timezone values, journals, schedule email methods, and unsupported classes. Validate with independent vCard/iCalendar/vJournal parsers, raw-field retention, exact/partial statuses, mixed-folder counts, and updates to [special email items](07-embedded-and-special-email-items.md), [storage](09-storage-and-interoperability.md), [the matrix](10-parity-matrix.md), and the source ledger.
