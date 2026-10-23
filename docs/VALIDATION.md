# Validation

## Scope

Host tests validate batch math, transaction deltas, cancellation, one-shot dispatch, and resource changes. `scripts/verify_game.py` checks a supplied NSO without writing extracted files. Neither is an in-game test.

The executable is Engage 2.0.0, Build ID `8C08B9719E085F91847B5E0F935D9488`. Save format/version codes such as `0x130` are not the executable's display version.

## Native integration

- `OnDecideSynthesis` receives and retains the original single-meld costs.
- Only during that synchronous call may `YesNoDialog.CreateBind` substitute the batch summary. The existing buttons and callbacks remain unchanged. The two-line summary shows the ring name once, source/result ranks and counts, and total fragment cost. Ring names come from the game; the fragment label follows `App.Language.GetLang`, not system locale or voice language. All 12 game text-language IDs are covered, with English fallback for unknown IDs.
- The menu's affirmative callback arms the plan. A plan cannot run unless its batch summary was successfully supplied to the native dialog.
- The confirmed plan survives the original menu destruction, then transfers once to `ExecuteRingMerge`.
- `SynthesisRingExecute.Do` uses its trampoline once per transaction. Its return value is accumulated as a new-acquisition flag, not used as a success test.
- Before each transaction, native stock queries and `CanAdd(next_rnid, 1)` run again. Native `CanAdd` accounts for the pool's 700-entry limit. The native add-before-subtract ordering is preserved.
- Inventory entries are looked up transiently. Only immutable database `RingData` is retained during the synchronous loop, allowing verification after the last source inventory entry disappears.
- A mismatch stops the loop, without retries, extra single transactions, or speculative rollback.
- `TitleBar.UpdateFooterValues` at `0x021EF170` remains in the original outer flow. The native result/demo sequence is not skipped or looped.
- All hook entries, native helpers, display version, and the mapped GNU Build ID are checked before hooks are installed. The title ID remains package metadata; startup does not call Skyline's unavailable `get_program_id` export. No cheat code caves are used.

## Cobalt interface compatibility

The ABI contract in `tools/cobalt-imports.json` records the providers of the plugin's runtime imports. It was cross-checked against the official Cobalt 1.31.0 `subsdk9` and the supplied Engage 2.0.0 modules. Only public symbol names and module fingerprints are stored; no executable bytes are included.

`verify_nro.py` follows both PLT and ordinary ELF RELA relocations in the actual NRO. Every referenced undefined symbol must have a known provider in that contract. This runs before packaging locally and in CI, including release builds. Non-relocated declarations such as Skyline's `__nro_header_start` are not loader dependencies.

Version 0.1.0 incorrectly depended on `get_program_id`, which Cobalt does not export. Its removal also lets the linker omit the unused Skyline information/NRO-hook modules and their unsupported imports. Regression tests reject all four missing Skyline interfaces and prevent incompatible artifacts from being packaged.

Passing this check establishes interface availability, not end-to-end game behavior. The pending in-game acceptance checks still apply.

The startup smoke job then executes the compiled NRO's `main` under Unicorn 2.1.4 with modeled SDK/Cobalt calls and synthetic executable fingerprints. It checks that the supported image returns normally after registering exactly nine hooks. Wrong display version, Build ID, instructions, or segment layout must return normally without installing hooks. Unexpected native calls, null jumps, or non-returning initialization fail CI. This sandbox does not load the game or execute the installed hooks.

To repeat the startup check without installing the mod:

```sh
uv run --with unicorn==2.1.4 python scripts/smoke_startup.py /path/to/fee_batch_ring_meld.nro
```

## Local checks

```sh
cargo test --locked
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/verify_game.py /path/to/extracted/main
cargo skyline build --release
python3 scripts/verify_nro.py target/aarch64-skyline-switch/release/libfee_batch_ring_meld.nro
```

## Pending in-game acceptance

The user confirmed working batch melding in version 0.1.1. This does not independently establish every edge case below or the rendering of later localized confirmation text.

Do not install this development build into a user's game without approval. Use a separate test save; do not overwrite the primary save.

- C→B, B→A, A→S; multiple groups, leftovers, one group, insufficient resources.
- Fragment-limited batches and insufficient fragments.
- Result counts 98 and 99; equipped source/result copies; full native pool.
- Previously obtained results (native false return) and first acquisition.
- Cancel confirmation, change selection, repeat melding, exit/re-enter the menu.
- One result presentation, correct footer/menu refresh, and normal sequence completion.
- Save/reload and reparse materials, outputs, fragments, acquisition records, and equipped owners. Unrelated records must remain unchanged.
- An active fragment-refill cheat must not extend the confirmed count. Temporarily disable that cheat with the user's agreement when verifying exact persistent fragment deductions.
- Unsupported executable or conflicting preinstalled hooks must leave original behavior unchanged.

The compact localized confirmation needs in-game visual verification. Edge cases, exact animation behavior, and save/reload comparisons remain pending unless explicitly tested.
