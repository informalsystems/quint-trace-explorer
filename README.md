# Quint ITF Trace Explorer

A terminal UI tool for exploring Quint/Apalache traces in the Informal Trace Format (ITF).

This is a toy, vibe-coded project. We advise you to use agents to change the code if changes are needed. Vibe-coded PRs are welcome if the feature/fix is clearly described and easy to validate.

## Overview

An interactive CLI tool to navigate, inspect, and debug ITF traces produced by Quint, Apalache and ([soon](https://github.com/tlaplus/tlaplus/compare/master...bugarela:tlaplus:gabriela/dump-to-itf?expand=1)) TLC.

The main goal is to make it easier to see what has changed from one state to another in a trace. We optimize the usage of available space (vertical and horizontal) to best show the changes, collapsing sub-trees that are unchanged (unless there is spare space).

## Requirements

- Rust 1.70 or later (2021 edition) - not needed if using Nix
- A terminal emulator with Unicode support

## Installation

### Using Nix

If you have Nix with flakes enabled:

```bash
nix run github:informalsystems/quint-trace-explorer -- <trace-file.itf.json>
```

### Using Cargo

```bash
# Install from source
git clone https://github.com/informalsystems/quint-trace-explorer.git
cd quint-trace-explorer
cargo build --release

# Run the tool
./target/release/quint-trace-explorer <trace-file.itf.json>
```

## Usage

### Generating ITF Traces

First, generate an ITF trace from your Quint specification using one of these commands:

```bash
# Generate a trace with a single random execution
quint run file.qnt --max-samples=1 --out-itf=trace.itf.json
```

All these commands support ITF output:
``` bash
quint run file.qnt --out-itf=trace.itf.json

quint test file.qnt --out-itf=trace.itf.json

quint verify file.qnt --out-itf=trace.itf.json
```

### Exploring Traces

Basic usage:

```bash
quint-trace-explorer <path-to-trace.itf.json>
```

If running from the repository (not installed):

```bash
cargo run -- <path-to-trace.itf.json>
```

Try it with one of the included examples:

```bash
# Using installed binary
quint-trace-explorer examples/tendermint.itf.json

# Or from repository
cargo run -- examples/clock.itf.json
cargo run -- examples/consensus.itf.json
```

Once running, use the keyboard navigation (see below) or your mouse to explore states and inspect values.

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

