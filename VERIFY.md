# Reproducible build & bytecode verification — pair-equality

This commit pins everything needed to rebuild `pair_equality.wasm`
**byte-for-byte**, so the deployed alkane can be verified from source
(explorer "Verify from source" flow).

## Deployed artifact

| Field | Value |
|---|---|
| Alkane | `[4:9200]` (mainnet, CREATERESERVED `[3,9200,0]`) |
| File | `target/wasm32-unknown-unknown/release/pair_equality.wasm` |
| Size | 169,973 bytes |
| sha256 | `b7e9335b36a20393aad8f80c2293c083ac19083fdf851fa91e49901cbe4df6ae` |

## Build

Prerequisites: Rust `1.86.0` (pinned by `rust-toolchain.toml`, installed
automatically by rustup) and `protoc` on PATH (set `PROTOC=/path/to/protoc`
if not auto-detected).

```bash
cargo build --release --target wasm32-unknown-unknown -p pair-equality
sha256sum target/wasm32-unknown-unknown/release/pair_equality.wasm
# -> b7e9335b36a20393aad8f80c2293c083ac19083fdf851fa91e49901cbe4df6ae
```

## What is pinned, and why

1. **`alkanes-rs` git deps pinned to rev `888f4fe6`** (workspace `Cargo.toml`
   + the `test-utils` dev-dependency in `pair-equality/Cargo.toml`).
   Unpinned, cargo resolves alkanes-rs HEAD, where the `metashrew-support`
   crate no longer exists (moved to the metashrew repo) — the build breaks
   and is not reproducible over time.
2. **`Cargo.lock` committed** (removed from `.gitignore`). Locks every
   transitive dep; without it newer `darling`/`serde_with`/`time` releases
   require rustc ≥ 1.88 and change the output bytes.
3. **`rust-toolchain.toml`** pins rustc `1.86.0` + the wasm target — the
   compiler version is part of the byte-exact output.
4. **Two mechanical API-drift patches in `pair-equality/src/lib.rs`**
   (the crate predates rev `888f4fe6`; no logic/ABI change):
   - `AlkaneResponder` no longer has an `execute` trait method
     (`declare_alkane!` generates dispatch) → the redundant
     `impl ... { fn execute }` became an empty `impl AlkaneResponder for
     EqualityPredicateAlkane {}`.
   - `observe_initialization` became ambiguous (the runtime trait gained a
     default method with byte-identical logic) → disambiguated to
     `EqualityPredicate::observe_initialization(self)`.

## ABI (unchanged)

- opcode `0` — `Initialize`: one-shot (`/initialized` guard; second call
  reverts "Contract already initialized").
- opcode `7` — `Filter { sequence_left, amount_left, sequence_right,
  amount_right }`: requires exactly 2 incoming alkanes; positional match of
  `[2:sequence_left] @ amount_left` and `[2:sequence_right] @ amount_right`;
  match → forward, mismatch → revert (runtime refunds via refund pointer).
