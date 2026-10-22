# FEE Batch Ring Meld

Batch Bond Ring melding for **Fire Emblem Engage 2.0.0**, using Cobalt.

Select a ring and rank in the original melding menu. A single confirmation melds as many copies of that selection as the available materials, Bond Fragments, and inventory capacity permit. Each result costs the normal amount. Equipped rings are not consumed, and results are not automatically melded into the next rank.

The original menu and one original result presentation are retained. No hotkeys, quantity dialog, configuration, or file logging are added.

## Development

This repository is under development. Static executable checks, host tests, and a Switch build are separate from in-game validation. No release is published or installed automatically.

Supported executable Build ID: `8C08B9719E085F91847B5E0F935D9488`.

```sh
cargo fmt --check
cargo test --locked
cargo skyline build --release
```

## Installation layout

Requires [Cobalt](https://github.com/Raytwo/Cobalt). The intended package contains only:

```text
engage/mods/fee_batch_ring_meld/fee_batch_ring_meld.nro
```

### Nintendo Switch

Install Cobalt following its instructions, then place the `engage` directory at the SD card root.

### Emulators

Install Cobalt for the emulator, then place `engage` beside Cobalt's existing mod directory as described by its installation guide. Do not put this plugin in the FETH Skyline plugins directory.

## References

- [Cobalt](https://github.com/Raytwo/Cobalt) and its [plugin development hub](https://github.com/DivineDragonFanClub/plugin_hub).
- [engage-il2cpp](https://github.com/DivineDragonFanClub/engage-il2cpp/tree/0.1.0) for function signatures, cross-checked against the supplied executable. No game binaries or assets are included.

## License

[MIT](LICENSE) License © [jinghaihan](https://github.com/jinghaihan)
