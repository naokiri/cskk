# Henkan Trigger Extension: Configurable Composition Triggers

## Overview

In standard SKK, the capital letters A–Z trigger composition mode (▽ mode /
PreComposition). When you type `Shift+a`, the OS resolves the keysym to `XK_A`
and cskk uses that to enter PreComposition and insert `あ`.

This extension makes the trigger set **fully configurable in the rule file**.
Rule authors list A–Z explicitly, and can add any other keysym (such as
`quotedbl` for `Shift+'`) as an additional trigger.

## Rule File Syntax

Add an `[options]` section after `[metadata]` and place `composition_triggers`
inside it:

```toml
[metadata]
name = "default"
description = "My typing rule"

[options]
composition_triggers = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
    "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"
]

[conversion]
a = ["", "あ"]
# ...
```

Keysym names follow the xkbcommon conventions — the same names used in
`[command]` bindings (e.g. `"A"`, `"quotedbl"`, `"at"`).

If `[options]` or `composition_triggers` is omitted, it defaults to an empty list
and no key triggers composition mode. The built-in default and AZIK rules both
include A–Z.

### Adding a Symbol Trigger

To make `Shift+'` (keysym `quotedbl`) enter PreComposition and insert `っ`:

```toml
[options]
composition_triggers = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
    "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
    "quotedbl"
]

[conversion]
# ...
quotedbl = ["", "っ"]
```

The rule **must** have a `[conversion]` entry for the keysym so cskk knows
which kana to insert when the trigger fires.

## How It Works

### Keysyms Encode the Shift State

The OS resolves Shift before cskk sees the key event:

| Key pressed | Keysym sent to cskk | Modifier |
|-------------|---------------------|----------|
| `a`         | `XK_a`              | none     |
| `Shift+a`   | `XK_A`              | SHIFT    |
| `'`         | `XK_apostrophe`     | none     |
| `Shift+'`   | `XK_quotedbl`       | SHIFT    |
| `"` (dedicated key) | `XK_quotedbl` | none |

Because the keysym already encodes which character the user intended, cskk
checks only `composition_triggers.contains(keysym)` — no separate SHIFT inspection
is needed.

### Trie Lookup for the Kana

When a trigger fires, cskk looks up the kana to insert:

- **A–Z triggers**: `XK_A` → looks up `XK_a` in the conversion trie (lowercased).
- **Symbol triggers**: `XK_quotedbl` → looks up `XK_quotedbl` unchanged.

This means a conversion entry like `a = ["", "あ"]` is used for both direct `a`
input (direct kana conversion) and triggered `A` input (PreComposition + あ).
Symbol entries like `quotedbl = ["", "っ"]` are used for triggered `quotedbl` input
only, since the raw `quotedbl` path is skipped for listed triggers.

### Raw Path Suppression for Symbol Triggers

For letter triggers (A–Z), the raw conversion path naturally misses because no
rule has an uppercase `A` key. For symbol triggers, the raw key and the lookup
key are the same (`XK_quotedbl` in both cases). Without suppression, cskk would
convert `っ` directly without entering PreComposition.

The engine suppresses raw-path branches for non-letter keysyms that appear in
`composition_triggers`, ensuring the trigger path fires instead.

## Limitations

- **No range syntax.** The list must enumerate each keysym name individually.
  `"A-Z"` is not a shorthand; write all 26 letters explicitly.
- **Conversion entry required.** A trigger keysym with no `[conversion]` entry
  will enter PreComposition but insert nothing, then wait for further input.
- **No "un-shift" mapping.** cskk does not know your keyboard layout, so it
  cannot derive which key `XK_quotedbl` came from. If you have a dedicated `"`
  key it produces the same `XK_quotedbl` keysym as `Shift+'` and is treated
  identically.
- **Rule is the authority.** Removing a keysym from `composition_triggers` means it
  never triggers composition mode, regardless of the engine version.

## Example: Custom Rule with Symbol Trigger

```toml
[metadata]
name = "custom"
description = "Default rule plus quotedbl trigger"

[options]
composition_triggers = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
    "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
    "quotedbl"
]

[conversion]
a   = ["", "あ"]
# ... other kana rules ...
quotedbl = ["", "っ"]

[command]
# ... command bindings ...
```

With this rule:
- `A` → `▽あ` (PreComposition, standard SKK behavior)
- `Shift+'` or bare `quotedbl` keysym → `▽っ` (PreComposition with っ)
- `Shift+2` (`XK_at`) → `@` output directly (not in triggers)
- Plain `'` (`XK_apostrophe`) → direct output (not in triggers)
