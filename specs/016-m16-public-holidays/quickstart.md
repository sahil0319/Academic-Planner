# Quickstart: M16 manual checks (human)

The unit and integration tests never touch the network. The real fetch needs a real network, so
these checks are for the human. Use a scratch database, never the real one:

```sh
ACADEMIC_PLANNER_DB="$PWD/target/scratch/demo.sqlite" cargo run -p app
```

1. **Setup.** Create a term that covers today (Settings -> Term). A term is needed; with no active
   term nothing is fetched (R77).
2. **Default country.** Open Settings -> Public holidays. The country shows your OS region if it is
   in the list, else "Off" (R76). Status reads "Updated today" after a moment.
3. **Dashboard.** Go to the Dashboard. A public holiday in the visible month has the teal fill; the
   legend has "Public holiday"; selecting the day shows "Public holiday: {name}" (R78).
4. **Switch country.** Pick another country (for example Bangladesh, then United Kingdom). The status
   shows loading, then "Updated today", and the list changes (R77).
5. **Add selected.** Tick two dates, click "Add selected as holidays". A toast says "Added 2
   holidays."; the dates now appear in the Holidays card and sessions on them are gone (R79, R6).
   Tick one of them again plus a mid exam day: the toast says "... 1 skipped, already a day off."
6. **Off.** Pick "Off": no request, the list is empty, the cached teal fills disappear.
7. **Retry.** Turn the network off, switch to a country you have not loaded, wait about 10 seconds.
   The status reads "Couldn't load public holidays" with a Retry button; any cached rows stay.
   Turn the network on and click Retry: the status becomes "Updated today".
8. **Calendar ids.** For any country that stays on "Couldn't load public holidays" with the network
   on, its id in `crates/app/data/holiday-calendars.json` is wrong (see D40 in decisions.md for
   the unverified list). Check one with
   `curl -sI "https://calendar.google.com/calendar/ical/<url-encoded id>/public/basic.ics"`.
9. **Backup.** Back up, restore: the country choice returns; cached public holidays are not in the
   file (R80).
