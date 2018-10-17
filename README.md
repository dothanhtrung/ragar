# ragar

A game like agar.io written in Rust with ggez

## Build
```sh
cargo build --release
```

## Run
### Host a server
```sh
./target/release/ragar_server
```

### Run game
```sh
./target/release/ragar
```

# TODO
- Virus
  - Draw virus with spiny.
- Split ragarman
  - Split when press `Space`.
  - Run faster.
  - Blow when eat virus.
  - Fusion.
- Game's configurable.
- Optimize
  - Draw.
  - Check if food is eaten.
- Start screen.
- Change connection to TCP.