# Reproducible build & bytecode verification — pair-equality-generic

Companion to the root [`VERIFY.md`](../VERIFY.md) (compact predicate, `[4:9200]`).
Everything about the toolchain pinning is identical; only the crate and the
deployed slot differ. **Read the "Build host" section — it is the one place
this artifact departs from a clean reproducible-build story.**

## Deployed artifact

| Field | Value |
|---|---|
| Alkane | `[4:9201]` (mainnet, CREATERESERVED `[3,9201,0]`) |
| Commit tx | `601871178e2e353d02e7233195ef6810737ae3e4b907e899e31d9357f35e33ec` |
| Reveal tx | `91f0e2cf22e0da1986193e783f86151b702d036588c67f451ed1a9308e584ec0` |
| Block | 959172 (2026-07-22) |
| File | `target/wasm32-unknown-unknown/release/pair_equality_generic.wasm` |
| Size | 171,241 bytes |
| sha256 | `a4074940d3b411aef3ba5d3bcacec5435ec5546e8a1d9da2596ab11a46899ff7` |

**On-chain identity is proven**: `metashrew_view getbytecode` for `[4:9201]`
returns exactly these 171,241 bytes with exactly this sha256 — byte-identical
to the artifact the devnet e2e suite exercised. Reproduce with:

```bash
alkanes-cli --jsonrpc-url https://mainnet.subfrost.io/v4/subfrost \
  build-info verify --at 4:9201 ./pair_equality_generic.wasm
```

## ABI

Declared in `contract.wit` + `alkanes.toml`; dispatch generated at build time
by `alkanes-wit-build` (`build.rs`). Opcodes and error strings are deliberately
IDENTICAL to the compact contract (flex, 2026-07-23: "same everything though,
same error strings etc").

| Opcode | Func | Behavior |
|---|---|---|
| 0 | `initialize` | One-shot (`observe_initialization`; re-init reverts "Contract already initialized"). |
| 7 | `filter(block-left, sequence-left, amount-left, block-right, sequence-right, amount-right)` | Requires exactly 2 incoming alkanes; positional match of `[block_left:sequence_left] @ amount_left` and `[block_right:sequence_right] @ amount_right`; match → forward, mismatch → revert. WIT ints widen to u128 (cellpack words). |

The only semantic difference from the compact contract: each leg's id is taken
from the calldata in full instead of being reconstructed as `{block: 2, tx:
sequence}`. That reconstruction is what made the compact predicate reject
frBTC `[32:0]` (mainnet settlement `d2c3291b…`, block 959087 — refunded).

## Build

Prerequisites, identical to the compact contract: Rust `1.86.0` (pinned by
`rust-toolchain.toml`), `protoc` on PATH, and **clang 16.0.0 from wasi-sdk
release 20** as the C compiler for the wasm target (`secp256k1-sys 0.10.1`
compiles vendored C through the `cc` crate; a different clang changes the
artifact hash):

```bash
export CC_wasm32_unknown_unknown=/path/to/wasi-sdk-20.0/bin/clang
cargo build --release --locked --target wasm32-unknown-unknown -p pair-equality-generic
```

The `producers` custom section of the deployed artifact reads
`rustc 1.86.0 (05f9846f8 2025-03-31)` + `clang 16.0.0`, confirming both pins.

## ⚠️ Build host — why this artifact is not sandbox-reproducible

**The deployed `[4:9201]` was built on Windows.** rustc bakes dependency paths
into panic strings, and on Windows those are backslash-separated Windows paths:

```
C:\Users\<user>\.cargo\git\checkouts\alkanes-rs-4aece39342c88293\ad27105\crates\alkanes-runtime\src\runtime.rs
C:\Users\<user>\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\anyhow-1.0.104\src\error.rs
```

The BuildInfo verifier reproduces builds in a **Linux** container
(`toolchain.os_image`), and rustc on Linux cannot emit those bytes — the path
separators come from the host OS, so no `HOME` value or `--remap-path-prefix`
setting reconstructs them. A Linux rebuild of this exact commit, with the same
rustc and the same wasi-sdk clang, produces a *different* artifact:

| | deployed (Windows) | Linux rebuild |
|---|---|---|
| size | 171,241 | 171,331 |
| sha256 | `a4074940d3b411ae…` | `1d2c9f1c995de171…` |

Section-level diff (everything else is byte-identical, including `producers`):

| section | delta | note |
|---|---|---|
| `data` | +64 | the embedded path strings |
| `custom:name` | +20 | same, in the name section |
| `code` | +4 | data-segment offsets shift |
| `elem` | +2 | |
| `type` / `import` / `table` / `memory` / `export` / `producers` / `target_features` | 0 | identical bytes |

**The Linux build itself IS byte-deterministic, and `HOME` is the variable**:

| build | HOME | sha256 | size |
|---|---|---|---|
| A | `/home/u/bihome` | `1d2c9f1c995de171…` | 171,331 |
| B (clean rebuild) | `/home/u/bihome` | `1d2c9f1c995de171…` ✅ same as A | 171,331 |
| C | `/home/u/bihome2` | `46e6263a1ec95a38…` ❌ differs | 171,325 |

B is a from-scratch rebuild (fresh source tree, fresh `CARGO_HOME`, no `target/`)
and lands on A's hash exactly; only changing `HOME` moves it. That is precisely
what `BuildInfo.environment.home` pins — so a Linux-built deployment of this
crate verifies byte-exact in the explorer sandbox, while a Windows-built one
never can.

Consequence: the honest verdict for `[4:9201]` against a Linux verifier is a
**mismatch on bytes, with the source relationship still auditable** — the
producers section, every structural section, and the on-chain-vs-local identity
above all check out; only the path strings differ. To get a `reproducible`
verdict the crate must be rebuilt and redeployed from a Linux host with a
pinned `HOME`.

### Known gaps in the tooling (reported upstream)

1. `buildinfo::wasm::reverse`'s `PATH_MARKERS` only match POSIX prefixes
   (`/home/`, `/Users/`, `/root/`, `.cargo/registry/src/`). A Windows-built
   artifact matches none of them, so `reverse` reports `home: None` and — because
   the rustc-internal `/rustc/<hash>/…` paths ARE present — sets
   `remapped: true`, i.e. it reads a Windows build as an already-path-remapped
   one. Suggested fix: also scan for `[A-Z]:\\` prefixes and report the host OS.
2. `c_toolchain.source.method` has no `wasi-sdk` variant (`apt` |
   `llvm-release` | `homebrew-bottle`), although wasi-sdk is what both predicate
   deploys used for the C leg.

## Behavioral proof

subfrost-app's orbitals devnet e2e suite, run against THIS binary
(`__tests__/devnet/orbitals/settlement-frbtc-generic.test.ts`): a REAL frBTC
wrap (opcode 77 to the GET_SIGNER address) settles orbital → buyer and frBTC
`[32:0]` → seller through the 9-word generic calldata — the SUCCESS path, not
the refund path the compact predicate took on mainnet. The compact-path suite
(`settlement-success.test.ts`, 2/2) is unaffected.
