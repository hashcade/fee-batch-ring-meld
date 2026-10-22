# FEE Batch Ring Meld

Batch Bond Ring melding for **Fire Emblem Engage 2.0.0**, using Cobalt.

Select a ring and rank in the original melding menu. A single confirmation melds as many copies of that selection as the available materials, Bond Fragments, and inventory capacity permit. Each result costs the normal amount. Equipped rings are not consumed, and results are not automatically melded into the next rank.

The original menu and one original result presentation are retained. No hotkeys, quantity dialog, configuration, or file logging are added.

## Development

The native menu hooks, transaction engine, and release tooling are implemented. Host tests, executable checks, and the Switch build are verified; **in-game validation is still pending**. See [validation coverage](docs/VALIDATION.md).

Supported executable Build ID: `8C08B9719E085F91847B5E0F935D9488`.

```sh
cargo fmt --check
cargo test --locked
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo skyline build --release
python3 scripts/verify_nro.py target/aarch64-skyline-switch/release/libfee_batch_ring_meld.nro
```

## Installation layout

Requires [Cobalt](https://github.com/Raytwo/Cobalt); the import contract is verified against **Cobalt 1.31.0** with Engage 2.0.0. The intended package contains only:

```text
engage/mods/fee_batch_ring_meld/fee_batch_ring_meld.nro
```

### Nintendo Switch

Install Cobalt following its instructions, then place the `engage` directory at the SD card root.

### Emulators

Install Cobalt for the emulator, then extract `engage` into the emulator's **virtual SD card root**. Cobalt plugins do not go in the emulator's per-game mod folder or the FETH Skyline plugins directory. See [Cobalt mod management](https://github.com/Raytwo/Cobalt/wiki/Managing-your-Mods).

## Releases

The workflow follows the FETH mods: stable semantic versions, `VERSION` matched with Cargo, a verified NRO, an installable ZIP, checksums, generated release notes, and verification of the uploaded downloads.

NRO verification also rejects runtime imports unavailable in the supported Cobalt/game modules. Version 0.1.0 has a startup compatibility bug; use 0.1.1 or later.

```sh
python3 scripts/release.py --current
python3 scripts/release.py --bump patch
```

Release from a clean `main` synchronized with GitHub. The script pushes the release commit/tag atomically; CI owns validation, compilation, and publication, without repeating full local builds. Only run this after a release has been explicitly requested.

No configuration, logs, or save files are bundled. The confirmation summary uses localized game ring names and English action/cost labels.

## References

- [Cobalt](https://github.com/Raytwo/Cobalt) and its [plugin development hub](https://github.com/DivineDragonFanClub/plugin_hub).
- [engage-il2cpp](https://github.com/DivineDragonFanClub/engage-il2cpp/tree/0.1.0) for function signatures, cross-checked against the supplied executable. No game binaries or assets are included.

## License

[MIT](LICENSE) License © [jinghaihan](https://github.com/jinghaihan)
