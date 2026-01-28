# Quint ITF Trace Explorer

A terminal UI tool for exploring Quint/Apalache traces in the Informal Trace Format (ITF).

This is a toy, vibe-coded project. We advise you to use agents to change the code if changes are needed. Vibe-coded PRs are welcome if the feature/fix is clearly described and easy to validate.

## Overview

An interactive CLI tool to navigate, inspect, and debug ITF traces produced by Quint, Apalache and ([soon](https://github.com/tlaplus/tlaplus/compare/master...bugarela:tlaplus:gabriela/dump-to-itf?expand=1)) TLC.

The main goal is to make it easier to see what has changed from one state to another in a trace. We optimize the usage of available space (vertical and horizontal) to best show the changes, collapsing sub-trees that are unchanged (unless there is spare space).

## Demo

TODO: video

## ITF Format Reference

ITF is a JSON-based trace format. See [ADR-015](https://apalache-mc.org/docs/adr/015adr-trace.html) for full spec.

### Trace Structure

```json
{
  "#meta": { "source": "spec.qnt" },
  "vars": ["activeTimeouts", "msgBuffer", "system"],
  "states": [
    { "#meta": { "index": 0 }, "activeTimeouts": ..., "msgBuffer": ..., "system": ... },
    { "#meta": { "index": 1 }, ... }
  ],
  "loop": null
}
```

### ITF Value Types

| ITF Form                       | Meaning       | Quint equivalent    |
|--------------------------------|---------------|---------------------|
| `{ "#bigint": "123" }`         | Integer       | `123`               |
| `{ "#set": [x, ...] }`         | Set           | `Set(x, ...)`       |
| `{ "#map": [[k,v], ...] }`     | Map/Function  | `Map(k -> v, ...)`  |
| `{ "#tup": [x, ...] }`         | Tuple         | `(x, ...)`          |
| `{ "tag": "X", "value": v }`   | Variant       | `X(v)`              |
| `[1, 2, 3]`                    | Sequence/List | `[1, 2, 3]`         |
| `{ "field": v, ... }` (no `#`) | Record        | `{ field: v, ... }` |
| `"hello"`                      | String        | `"hello"`           |
| `true` / `false`               | Boolean       | `true` / `false`    |

---

## Navigation

### State Navigation

| Key       | Action                           |
|-----------|----------------------------------|
| `←` / `h` | Previous state                   |
| `→` / `l` | Next state                       |
| `g`       | Go to state (prompts for number) |
| `Home`    | First state                      |
| `End`     | Last state                       |

### Tree Navigation

| Key               | Action                            |
|-------------------|-----------------------------------|
| `↑` / `k`         | Move cursor up                    |
| `↓` / `j`         | Move cursor down                  |
| `Enter` / `→`     | Expand node under cursor          |
| `←` / `Backspace` | Collapse node (or jump to parent) |

### Other

| Key         | Action                          |
|-------------|---------------------------------|
| `/`         | Search/filter states            |
| `v`         | Toggle variable visibility menu |
| `q` / `Esc` | Quit                            |

