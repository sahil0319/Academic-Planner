# Quickstart: validating M8 12-Hour Times

## Automated

```sh
cargo test -p app t34_
cargo test -p app t35_
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
! grep -rnE '#[0-9a-fA-F]{6}|[0-9]+px' crates/app/ui --include=*.slint | grep -v theme.slint
```

Expected: T34 and T35 pass, the gate is green, and the grep prints nothing.

## Manual (human, about 5 minutes, `cargo run -p app` on a copy of real data or a fresh DB)

1. Dashboard: every class time and due time reads like "9:30 AM" or "1:30 PM".
2. Timetable week view: session times read "9:00 AM–10:30 AM". Open a session, Reschedule: the
   start and end fields are pre-filled as digits with the right toggle. Type `930`, pick PM, save:
   the moved session shows "9:30 PM".
3. Timetable Rules: slot rows are 12-hour. Add a slot with start `1300` → inline error. Start
   `9` AM, end `1030` AM → saves. Start `1030` AM, end `9` AM → core's error shows "10:30 AM" and
   "9:00 AM".
4. Courses → a course → Add makeup class: start `130` PM, end `3` PM → saves; session list shows
   "1:30 PM–3:00 PM".
5. Assessments: new assessment, due time `1159` PM → list shows "…, 11:59 PM". Edit it: field shows
   `1159` with PM.
6. Toggle by keyboard: Tab to AM/PM, press Space or Enter, and the selection moves.
7. Settings → Backup: the saved JSON still has `"start_time": "09:00"`-style 24-hour times; Export
   CSV keeps `HH:MM`.
