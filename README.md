# PEEK

**Point, Examine, Easily Know.**

PEEK is a client-side information overlay for [Horizon](https://playhorizon.dev/). Point at a block or entity and a compact panel tells you what it is.

## Features

- block names and item icons
- block-state properties, including crop age and orientation
- suggested tools for common block types
- entity names and health
- source namespace for blocks and entities
- no server-side installation required

## Compatibility

- Minecraft 26.3
- Horizon mod API 0.2
- Horizon build `0b14f8acfd97fa2c750e42795249dcc981178ca4`

## Building

Install the Horizon mod tooling, then run:

```sh
cargo horizon-mod update
cargo horizon-mod build
```

The resulting package is written to `target/horizon-mod/what_am_i_looking_at.hmod`.

## Installation

Copy the built `.hmod` file into the `horizon-mods` directory of your Horizon instance and enable PEEK in the launcher.

## License

[MIT](LICENSE)
