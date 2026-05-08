# printing-press-rs

A Rust-native port spike for the CLI Printing Press.

## Why this exists

The upstream Printing Press is written in Go and already does a lot. This spike explores whether a Rust implementation can preserve the same machine-level behavior while improving:

- startup latency
- memory discipline on large generation runs
- deterministic binary performance
- long-term maintainability for a stricter internal type model

## Current status

This is not feature-parity yet.

Right now it provides:
- the top-level command surface
- a shared error boundary
- a module layout that mirrors the Go machine
- placeholders for the major pipeline verbs

## Design rules

- keep the Rust port isolated under `rust-port/`
- use the Go codebase as the behavioral reference
- prefer deterministic parity checks before live integrations
- port subsystems in vertical slices, not a giant rewrite

## Run

```bash
cargo run --manifest-path rust-port/Cargo.toml -- --help
cargo run --manifest-path rust-port/Cargo.toml -- generate notion
```
