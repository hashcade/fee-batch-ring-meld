# Project guidelines

- Write source, comments, scripts, and documentation in English.
- Keep changes in atomic commits. Do not publish a version or install a mod unless requested.
- Follow the engineering conventions of the FETH mods, not their game-specific loader or offsets.
- Scope verification to the change. Use focused local tests and one affected build; CI owns full regression checks. Release scripts must not repeat full local builds.
- Never modify a user's save or game installation while developing. Read-only analysis of supplied executable files is permitted. Never commit game binaries, keys, saves, or extracted assets.
- Run all native transactions on the game's original menu thread. Preserve the original confirmation and outer sequence.
- Check the supported game fingerprint and instruction signatures before installing any hook. Fail closed on mismatches.
- Preserve per-transaction costs. Snapshot the batch limit, recheck each transaction, and verify inventory deltas. Never consume equipped rings or chain to another rank.
- After any attempted transaction, never fall back to another original transaction or guess a rollback.
- Do not add configuration, file logging, hotkeys, or extra menus without a user request.
- Document hardware installation before emulator installation. Use standard MIT attribution to jinghaihan; identify external references accurately.
