# Windows text-context boundaries

- UI Automation reports an editable field only when it has focus, is enabled and onscreen, has a valid bounding rectangle, is an Edit control, and explicitly reports `Value.IsReadOnly = false`.
- A read-only Chrome Document can expose `TextEditPattern`, so that pattern is not proof that the control is editable. Missing or failed properties are treated as unknown, and input contents are never read.
- Document controls, rich text, `contenteditable`, custom-drawn editors, and editors without `ValuePattern` may be missed. This is a deliberate reduction in scope; it does not add application allowlists, cursor heuristics, or protocol fields.
- The `InputPane` channel remains available, so this change alone cannot guarantee that older clients will never show the keyboard incorrectly. Pair it with Moonlight V+ #595's manual-keyboard policy.
- Merging the source does not update installed applications. Normal integration and release are still required, and host caching, focus identity, and all `InputPane` layouts do not yet have complete coverage.

Regression tests in `text_context.rs` cover writable, read-only, and unknown Edit controls as well as Document and other control types. The historical Chrome sample is a regression fixture, not proof of complete application or live-stream coverage.
