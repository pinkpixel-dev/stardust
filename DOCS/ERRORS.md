# Errors

Failed approaches and difficult bugs that are worth remembering.

Log a failure when it took more than two attempts, the root cause was somewhere other than the symptom, the behavior is environment-specific and will recur, or a reasonable next approach would fail the same way. This is not a bug tracker. Ordinary bugs found and fixed quickly do not belong here.

Format:

```md
## YYYY-MM-DD

### Note: <the trap, stated as a fact>

What did not work: <the approach and the actual observed failure>

What worked instead: <the fix, specific enough to repeat>

Note for next time: <the general lesson, one sentence>
```

Title the trap, not the symptom, so someone hitting it again can find the entry. If the same trap returns, update the existing entry rather than adding a second one.

## 2026-10-03

### Note: A scrollable passes the viewport height down as a minimum, and `flex_row` hands it to every child

What did not work: Putting `flex_row(cards)` directly inside `widget::scrollable`. Every card stretched to the full window height, even though the button, tooltip, and card column were all `Shrink`. Hover outlines ran to the bottom of the window and tooltips with `Position::Bottom` showed up there too.

What worked instead: Wrapping the `flex_row` in `widget::column::with_capacity(1)` before passing it to the scrollable. iced's scrollable builds the child limits from `limits.min()`, so the minimum height is the viewport. `flex_row` lays out each child with its own limits unchanged, and a `Shrink` button resolves up to that minimum. A column lays out its children with `Size::ZERO` as the minimum, which resets it.

Note for next time: When a `Shrink` widget inside a libcosmic `flex_row` comes out too big, check what minimum the parent is passing down before looking at the widget itself.
