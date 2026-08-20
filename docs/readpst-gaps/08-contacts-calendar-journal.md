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
