# Dr. Rustario vs. Rustris

Two falling block games on one engine, written in SDL2 and Rust for fun:

* **Dr. Rustario** — a Dr. Mario clone (NES, SNES, N64 and modern themes)
* **Rustris** — Tetris with the guideline ruleset (Game Boy, NES, SNES and modern themes)

Each player picks a game, so one player can clear viruses while the other stacks tetrominoes,
trading garbage across games. A player can also pick a playlist — *rustris then dr. rustario* —
and alternate between the games one stage (a level, a bottle) at a time, carrying their score.

Only core SDL2 is required: images, fonts and audio mixing are handled in Rust
(`image`, `ab_glyph`, `symphonia` + a small mixer on SDL's audio callback).

## Layout

| crate | what it is |
|---|---|
| `engine/` | everything that is not game rules: SDL app shell, menus, high scores, config, input, rendering (sprite sheets, themes, fonts, particles, animations), audio mixer, the match session |
| `dr-rustario/` | Dr. Rustario's rules (bottle, pills, viruses) and theme data |
| `rustris/` | Rustris's rules (board, SRS, scoring, garbage), theme data and, behind the `ai` feature, its genetic/neural AI |
| `launcher/` | the `dr-rustario-vs-rustris` binary: picks games and options and runs a match |

A game implements `engine::game::Game` (a headless board of `Cell`s with game-private
`CellId`s, producing engine `GameEvent`s) and `engine::render::GameRender`; its themes are
data handed to the engine's `retro_theme` and `modern_theme` builders. Attacks between players
are a neutral strength plus game-private detail, so Dr. Rustario garbage keeps its colours
between two Dr. Rustario players and becomes random colours when it comes from Rustris.

## Building

Requires vcpkg to build.

```bash
cargo install cargo-vcpkg
cargo vcpkg build
cargo build --release
```

All resources are embedded into the binary. Add `--features ai` for the Rustris AI opponent,
demo mode and the `ga` training subcommand (`dr-rustario-vs-rustris ga [auto|survival|score|diagnose]`).

### macOS

The linker will fail to link SDL2 haptics. You will need to add the following to `~/.cargo/config.toml`:

```toml
[target.aarch64-apple-darwin]
rustflags = ["-C", "link-args=-weak_framework CoreHaptics"]
```

### Linux

```shell
# Fedora
sudo dnf install SDL2-devel

# Ubuntu/Debian
sudo apt install libsdl2-dev
```

Build with pkgconfig:

```shell
cargo build --release --no-default-features --features pkgconfig
```

### Retro handhelds

Dr. Rustario was built for [ArkOS](https://github.com/christianhaitian/arkos) on the
[Anberic rg353m](https://anbernic.com/products/rg353m) via `./build-aarch64-cross.sh`
(Docker, glibc <2.30). The `retro_handheld` feature still compiles but is no longer maintained.

## Config

Config and high scores are stored in yaml:

* Windows: `$HOME\AppData\Roaming\dr-rustario-vs-rustris`
* MacOS: `$HOME/Library/Application Support/dr-rustario-vs-rustris`
* Linux: `$XDG_CONFIG_HOME/dr-rustario-vs-rustris` or `$HOME/.config/dr-rustario-vs-rustris`

High scores are kept per game (`high_scores.dr-rustario.yml`, `high_scores.rustris.yml`) plus
`mixed` and `playlist` tables. Scores from the standalone games are not carried over.

Most of it you can ignore except:

### Video Mode

* `Window` (default) - note if your screen is not at least 720p then the game may not even load on first attempt.
    ```yaml
    video:
      mode: !Window
        width: 1280
        height: 720
    ```
* `FullScreen` - native fullscreen (recommended), note the game should scale to any weird resolution but was designed for 1080p & 4k.
    ```yaml
    video:
      mode: !FullScreen
        width: 1920
        height: 1080
    ```  
* `FullScreenDesktop` - fullscreen in windowed mode
    ```yaml
    video:
      mode: !FullScreenDesktop
    ```  

### Controls

Only keyboard controls are supported.

```yaml
input:
  menu:
    up: Up
    down: Down
    left: Left
    right: Right
    select: X
    start: Return
  player1:
    move_left: Left
    move_right: Right
    soft_drop: Down
    hard_drop: Up
    rotate_clockwise: X
    rotate_anticlockwise: Z
    hold: LShift
  player2: ~
  pause: F1
  next_theme: F2
  quit: Escape
```

All key names are defined in [engine/src/config.rs](engine/src/config.rs).

There are no default player 2 controls.

## Rustris AI

Built with `--features ai`.

The **ai** option on a Rustris main menu selects who plays:

* `off` — human players.
* `vs challenging` / `vs difficult` / `vs impossible` — in a 2-player match the AI plays as player 2 (who must be on
  Rustris) and is speed limited by pressing one key every 250 ms / 80 ms / instantly (see `AiDifficulty` in
  `rustris/src/game/rules.rs`).
* `demo` — the first player's board is played by the AI at full speed; their controls are disabled.

Only human players can enter the high score table.

For each piece, calculate all possible positions and calculate the cost of each, choose the position with the best cost.
Cost parameters:
* Closed holes (a gap that cannot be filled without clearing a line)
* Open holes (a gap under a tetromino that can be filled)
* Max height of the stack
* bumpiness (the amount that the line height changes from left to right)
* optimising for tetris:
   * bad: blocks in the right most column
   * bad: clearing less that 4 lines
   * good: clearing a tetris

Algorithm:

1. wait until a frame where a tetromino is spawned
2. calculate lowest cost for spawned tetromino
3. if none held: calculate lowest cost for next tetromino
   if one held: calculate lowest cost for held tetromino
4. take the tetromino and position with lowest cost
5. press hold if held or next tetromino was chosen
6. apply input sequence of chosen position
7. hard drop
8. repeat

TODO

* compare recordings for the same seed with different scores to figure out what is hapenning
* record seed in game recording
* record tetromino sequence in game recording to validate seed in gameplay
* genetic algorithm crossover should maybe also have a chance of averaging some weights of the parents as well as flat out crossover
* genetic algorithm speciation
   * define minimum distance between a species OR use k-means
   * classify the population into species
   * ensure we retain at least 2, 3, 4 of the best species per generation
* AI upscale the retro themes