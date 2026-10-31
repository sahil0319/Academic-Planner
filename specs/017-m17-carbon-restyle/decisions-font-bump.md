# Decisions: font size bump (overnight, router-approved options)

- Every type size token goes up by 1px: title 24 to 25, headline 16 to 17, numeral 24 to 25, body and body-emphasis 14 to 15, body-sm and labels 12 to 13. Weights stay 400 and 600.
- Line height comments grow with the sizes (headline 24px, body 22px, body-emphasis 20px, small 18px).
- `row-height` stays 40px, `input-height` and `nav-item-height` stay 32px: a 15px line is about 20px tall, so it fits the existing rows and controls; 44px rows would cost vertical space on the week view and dashboard.
- Fixed-width columns that hold text grow by about 7% so text does not elide earlier: weekday and code 96 to 104; date and time 144 to 160; room 128 to 136; status 240 to 256.
- `toast-width` 320 to 336 (4px grid). The toast text wraps, so only line length changes.
- `calendar-cell` stays 36px: the day number box is 36 - 3*4 - 4 = 20px, and a 15px line is about 20px, so it just fits (the Text is centred, not clipped).
- `appearance-segment` and `toast-close-size` hold icons only and stay.
- No Rust changes, no default window size change. `DESIGN.md` front matter and prose follow the tokens.
