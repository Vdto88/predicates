# Reproducible build & bytecode verification — pair-equality

This commit pins everything needed to rebuild `pair_equality.wasm`
**byte-for-byte**, so the deployed alkane can be verified from source
(explorer "Verify from source" flow).

## Deployed artifact

| Field | Value |
|---|---|
| Alkane | `[4:9200]` (mainnet, CREATERESERVED `[3,9200,0]`) |
| File | `target/wasm32-unknown-unknown/release/pair_equality.wasm` |
| Size | 170,813 bytes |
| sha256 | `c4f6dc800af7475746374e83a62285d34d8ed0034b4a171ddd415594bf362f4a` |

## Build

Prerequisites: Rust `1.86.0` (pinned by `rust-toolchain.toml` — same channel
alkanes-rs develop pins; installed automatically by rustup), `protoc` on
PATH (set `PROTOC=/path/to/protoc` if not auto-detected), and **clang 16.0.0
from wasi-sdk release 20** as the C compiler for the wasm target:

```bash
export CC_wasm32_unknown_unknown=/path/to/wasi-sdk/bin/clang
```

The C compiler is part of byte-exact reproducibility: `secp256k1-sys 0.10.1`
(pulled via alkanes-support → bitcoin) compiles vendored C into the final
wasm through the `cc` crate. A different clang version produces different
object code and therefore a different artifact hash. wasi-sdk 20's clang
defaults to `wasm32-unknown-wasi`; the `cc` crate retargets it to
`wasm32-unknown-unknown` automatically.

```bash
cargo build --release --target wasm32-unknown-unknown -p pair-equality
sha256sum target/wasm32-unknown-unknown/release/pair_equality.wasm
# -> c4f6dc800af7475746374e83a62285d34d8ed0034b4a171ddd415594bf362f4a
```

## ABI — declared in `contract.wit` (WIT), mapped in `alkanes.toml`

Per current alkanes-rs develop convention, the ABI is `pair-equality/contract.wit`
(interface) + `pair-equality/alkanes.toml` (name→opcode map); dispatch/entry
points are generated at build time by `alkanes-wit-build` (`build.rs`) — no
hand-rolled opcode enum.

| Opcode | Func | Behavior |
|---|---|---|
| 0 | `initialize` | One-shot (`/initialized` guard; re-init reverts "Contract already initialized"). |
| 7 | `filter(sequence-left, amount-left, sequence-right, amount-right)` | Requires exactly 2 incoming alkanes; positional match of `[2:sequence_left] @ amount_left` and `[2:sequence_right] @ amount_right`; match → forward, mismatch → revert (refund via refund pointer). WIT ints are widened to u128 by the codegen (cellpack words). |

## What is pinned, and why

1. **alkanes-rs git deps → rev `ad27105f`** (develop tip at pin time) — the
   current runtime + `alkanes-wit-build` codegen.
2. **`metashrew-support` → `kungfuflex/metashrew` rev `eca790ca`** (v10):
   alkanes-rs develop excludes the in-tree v9 metashrew crates and takes them
   from the metashrew repo; this is the exact rev alkanes-rs develop's own
   Cargo.lock resolves. The crate is referenced by the WIT-generated entry
   points (`to_arraybuffer_layout`/`to_passback_ptr`).
3. **`Cargo.lock` committed**, with `darling 0.21.3` / `serde_with 3.15.1`
   pinned to the same versions as alkanes-rs develop's lock — fresh
   resolution pulls 0.23/3.21 which require rustc ≥ 1.88 and break the 1.86
   toolchain pin.
4. **`rust-toolchain.toml`** pins rustc `1.86.0` + the wasm target — the
   compiler version is part of the byte-exact output.

## Behavioral proof

subfrost-app's orbitals devnet e2e suite (17/17) re-run against THIS binary:
happy-path settlement, swapped-calldata refund, forced-reject refund with
zero cross-leakage, buyer keeps full UTXO on reject, double-init guard.
Error strings are load-bearing for that suite and unchanged from the
original implementation.
