# Dr. Rustario

Dr. Mario clone written in SDL2 and Rust for fun.

## Building

Requires vcpkg to build.

```bash
cargo install cargo-vcpkg
cargo vcpkg build
cargo build --release
```

All resources are embedded into the binary.

## Config

Config is stored in yaml:

* Windows: `$HOME\AppData\Roaming\dr-rustario`
* MacOS: `$HOME/Library/Application Support/dr-rustario`
* Linux: `$XDG_CONFIG_HOME/dr-rustario` or `$HOME/.config/dr-rustario`

Most of it you can ignore except:

### Video Mode

* `Window` (default) - note if your screen is not at least 720p then rustris may not even load on first attempt.
    ```yaml
    video:
      mode:
        Window:
          width: 1280
          height: 720
    ```
* `FullScreen` - native fullscreen (recommended), note rustris should scale to any weird resolution but was designed for 1080p & 4k.
    ```yaml
    video:
      mode:
        FullScreen:
          width: 1920
          height: 1080
    ```  
* `FullScreenDesktop` - fullscreen in windowed mode
    ```yaml
    video:
      mode:
        FullScreenDesktop
    ```  

### Controls

Only keyboard controls are supported (I play this on a custom arcade cabinet with a programmable keyboard encoder).

```yaml
input:
  menu:
    up: Up
    down: Down
    left: Left
    right: Right
    select: Z
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
  quit: Escape
  next_theme: F2
```

All key names are defined in [src/config.rs](src/config.rs::KeycodeDef).

There are no default player 2 controls.

## TODO
* bg shadows
* themes
  * n64
    * virus pop
    * virus animation is a yoyo
  * modern
    * wii dr mario
    * wii bottle
    * font based game metrics like tetris effect
    * wii sound effects & music
    * particles
    * space background
