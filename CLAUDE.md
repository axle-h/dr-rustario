@README.md

## Layout

| crate | what it is |
|---|---|
| `engine/` | everything that is not game rules: SDL app shell, menus, high scores, config, input, rendering (sprite sheets, themes, fonts, particles, animations), audio mixer, the match session |
| `dr-rustario/` | Dr. Rustario's rules (bottle, pills, viruses) and theme data |
| `rustris/` | Rustris's rules (board, SRS, scoring, garbage), theme data and its genetic/neural AI |
| `launcher/` | the `dr-rustario-vs-rustris` binary: picks games and options and runs a match |

A game implements `engine::game::Game` (a headless board of `Cell`s with game-private
`CellId`s, producing engine `GameEvent`s) and `engine::render::GameRender`; its themes are
data handed to the engine's `retro_theme` and `modern_theme` builders. Attacks between players
are a neutral strength plus game-private detail, so Dr. Rustario garbage keeps its colours
between two Dr. Rustario players and becomes random colours when it comes from Rustris.