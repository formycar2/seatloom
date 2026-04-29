# NIMBUS-2026-04-28-foundation-scaffold-entrypoint-metadata-fix-delivery-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | fix |
| id | NIMBUS-2026-04-28-foundation-scaffold-entrypoint-metadata-fix-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-04-29 |
| version | v1 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-entrypoint-metadata-fix-v1.md`, `docs/coordination/acceptance/2026-04-28-lyra-nimbus-foundation-scaffold-acceptance.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-delivery-v1.md` |
| supersedes | - |
| tags | architecture, scaffold, rust, tauri, metadata, fix |

## 1. Scope completed

Closed the two remaining acceptance blockers on the foundation scaffold packet without expanding into runtime-engine work. This bounded fix packet re-verified the minimal Tauri binary entrypoint and normalized the original scaffold delivery artifact to the active `template+subtype` taxonomy.

## 2. Entrypoint fix

`src-tauri/src/main.rs` now exposes the smallest valid Tauri binary entrypoint:

```rust
fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to run SeatLoom Tauri shell");
}
```

No further runtime wiring was added. Command handlers remain stub-level and unmanaged in this packet by design.

## 3. Metadata fix

The original foundation scaffold delivery artifact now uses a machine-readable metadata table with the active taxonomy:

- `template | T3`
- `subtype | task`
- `status | delivered`

This removes the invalid `Subtype: delivery` header text and brings the artifact back into the `T3` allow-list defined in `docs/coordination/DOCUMENT_TEMPLATES.md` §5.

## 4. Changed files

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-delivery-v1.md` — normalized to explicit `template+subtype` metadata table using valid `T3/task`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-entrypoint-metadata-fix-delivery-v1.md` — new bounded-fix delivery artifact

## 5. Validation status

This packet is a static documentation-contract fix plus entrypoint re-verification.

Local compile verification remains unavailable on the current Nimbus seat:

```bash
$ source "$HOME/.cargo/env" && cargo check -p seatloom-core
bash: /Users/jyxc-dz-0100609/.cargo/env: No such file or directory

$ which cargo
cargo: not found
```

**Local compile verification deferred: cargo/rustc unavailable on the current seat environment.**

## 6. Blockers

- `ENV-001` — `cargo` / `rustc` are unavailable on the Nimbus seat, so no local compile pass was added in this bounded fix packet.

## 7. Evidence paths

- Acceptance review: `docs/coordination/acceptance/2026-04-28-lyra-nimbus-foundation-scaffold-acceptance.md`
- Fix packet: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-entrypoint-metadata-fix-v1.md`
- Normalized scaffold delivery: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-delivery-v1.md`
- Tauri entrypoint: `src-tauri/src/main.rs`
- Template taxonomy: `docs/coordination/DOCUMENT_TEMPLATES.md`
