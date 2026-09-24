[![Crates.io](https://img.shields.io/crates/v/deno_panic.svg)](https://crates.io/crates/deno_panic)

[Documentation](https://docs.rs/deno_panic) | [Web](https://panic.deno.com)

# `deno_panic`

`deno_panic` is a library for handling panics in Deno. It provides a way to
encode stack traces into URL-safe traces that can be decoded remotely using
debug information.

## Why does this exist?

Panics in Deno are rare—but when they happen, understanding them is crucial.
Including debug info directly in binaries makes them bulky. This tool lets users
symbolicate traces remotely, so Deno ship small binaries without losing
observability.

## Linux frame-pointer traces

The optional `frame-pointer` feature adds a `trace_frame_pointer()` walker on
x86-64 and AArch64 Linux. The existing libunwind-based `trace()` API remains
available, so enabling the feature is additive. The frame-pointer walker lets
release binaries remove `.eh_frame` and `.eh_frame_hdr` while retaining
remotely symbolicated panic traces.

Every frame between the panic hook and the panic site must preserve frame
pointers. Compile Rust code with `-Cforce-frame-pointers=yes` and rebuild the
standard library with the same flag (for example, with Cargo's `-Zbuild-std`).
Native code on the call path must likewise preserve frame pointers. Link with
`--no-eh-frame-hdr` before removing the unwind sections from the final ELF.

## Deployment

`panic.deno.com` runs on Cloud Run (project `panic-deno-com`, service `panic`,
region `us-central1`), built from the `Dockerfile` in this repo. A Cloud Build
trigger rebuilds and redeploys on every push to `main`; there is no GitHub
Actions deploy step.

Symbolicated traces and hit counters are stored in Deno KV. The database is
owned by the `panic-kv` app in the `deno` Deno Deploy org — that app only
exists to hold the database, since a KV database has to be attached to an app.
Do not delete it. The service reads `DENO_KV_DATABASE_URL` (the database's
connect URL) and `DENO_KV_ACCESS_TOKEN` (a Deploy token, from Secret Manager
secret `deno_kv_access_token`).
