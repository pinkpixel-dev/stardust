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
