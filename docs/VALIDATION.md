# Validation

## Scope

Host tests validate batch math, transaction deltas, cancellation, one-shot dispatch, and resource changes. `scripts/verify_game.py` checks a supplied NSO without writing extracted files. Neither is an in-game test.

The executable is Engage 2.0.0, Build ID `8C08B9719E085F91847B5E0F935D9488`. Save format/version codes such as `0x130` are not the executable's display version.

## Native integration

- `OnDecideSynthesis` receives and retains the original single-meld costs.
- Only during that synchronous call may `YesNoDialog.CreateBind` substitute the batch summary. The existing buttons and callbacks remain unchanged. The summary uses game-localized ring names and English action/cost labels.
- The menu's affirmative callback arms the plan. A plan cannot run unless its batch summary was successfully supplied to the native dialog.
- The confirmed plan survives the original menu destruction, then transfers once to `ExecuteRingMerge`.
- `SynthesisRingExecute.Do` uses its trampoline once per transaction. Its return value is accumulated as a new-acquisition flag, not used as a success test.
- Before each transaction, native stock queries and `CanAdd(next_rnid, 1)` run again. Native `CanAdd` accounts for the pool's 700-entry limit. The native add-before-subtract ordering is preserved.
- Inventory entries are looked up transiently. Only immutable database `RingData` is retained during the synchronous loop, allowing verification after the last source inventory entry disappears.
- A mismatch stops the loop, without retries, extra single transactions, or speculative rollback.
- `TitleBar.UpdateFooterValues` at `0x021EF170` remains in the original outer flow. The native result/demo sequence is not skipped or looped.
- All hook entries, native helpers, game ID, display version, and the mapped GNU Build ID are checked before hooks are installed. No cheat code caves are used.

## Local checks

```sh
cargo test --locked
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/verify_game.py /path/to/extracted/main
cargo skyline build --release
python3 scripts/verify_nro.py target/aarch64-skyline-switch/release/libfee_batch_ring_meld.nro
```

## Pending in-game acceptance

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

Runtime menu rendering, animation completion, and save/reload behavior remain unverified until these tests are actually performed.
