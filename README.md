[![Crates.io](https://img.shields.io/crates/v/deno_panic.svg)](https://crates.io/crates/deno_panic)

[Documentation](https://docs.rs/deno_panic) | [Web](https://panic.deno.com)

# `deno_panic`

`deno_panic` is a library for handling panics in Deno. It provides a way to encode
stack traces into URL-safe traces that can be decoded remotely using debug information.

## Why does this exist?

Panics in Deno are rare—but when they happen, understanding them is
crucial. Including debug info directly in binaries makes them bulky.
This tool lets users symbolicate traces remotely, so Deno ship small
binaries without losing observability.

