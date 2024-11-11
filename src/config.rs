use crate::game_input::GameInputKey;
use crate::menu_input::MenuInputKey;
use sdl2::keyboard::Keycode;
use sdl2::mixer::MAX_VOLUME;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::path::PathBuf;
use confy::ConfyError;
use serde::de::{Error, Visitor};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoMode {
    Window { width: u32, height: u32 },
    FullScreen { width: u32, height: u32 },
    FullScreenDesktop,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Config {
    pub video: VideoConfig,
    pub audio: AudioConfig,
    pub input: InputConfig,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MenuInputConfig {
    pub up: GameKey,
    pub down: GameKey,
    pub left: GameKey,
    pub right: GameKey,
    pub select: GameKey,
    pub start: GameKey,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GameInputConfig {
    pub move_left: GameKey,
    pub move_right: GameKey,
    pub soft_drop: GameKey,
    pub hard_drop: GameKey,
    pub rotate_clockwise: GameKey,
    pub rotate_anticlockwise: GameKey,
    pub hold: GameKey,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct InputConfig {
    pub menu: MenuInputConfig,
    pub player1: GameInputConfig,
    pub player2: Option<GameInputConfig>,
    pub pause: GameKey,
    pub quit: GameKey,
    pub next_theme: GameKey,
}

impl InputConfig {
    pub fn menu_map(&self) -> HashMap<Keycode, MenuInputKey> {
        HashMap::from([
            (self.menu.up.into(), MenuInputKey::Up),
            (self.menu.down.into(), MenuInputKey::Down),
            (self.menu.left.into(), MenuInputKey::Left),
            (self.menu.right.into(), MenuInputKey::Right),
            (self.menu.start.into(), MenuInputKey::Start),
            (self.menu.select.into(), MenuInputKey::Select),
            (self.quit.into(), MenuInputKey::Back),
        ])
    }

    pub fn game_map(&self) -> HashMap<Keycode, GameInputKey> {
        let mut result = HashMap::from([
            (self.quit.into(), GameInputKey::ReturnToMenu),
            (self.pause.into(), GameInputKey::Pause),
            (self.next_theme.into(), GameInputKey::NextTheme),
            (self.player1.move_left.into(), GameInputKey::MoveLeft { player: 0 }),
            (
                self.player1.move_right.into(),
                GameInputKey::MoveRight { player: 0 },
            ),
            (self.player1.soft_drop.into(), GameInputKey::SoftDrop { player: 0 }),
            (self.player1.hard_drop.into(), GameInputKey::HardDrop { player: 0 }),
            (
                self.player1.rotate_anticlockwise.into(),
                GameInputKey::RotateAnticlockwise { player: 0 },
            ),
            (
                self.player1.rotate_clockwise.into(),
                GameInputKey::RotateClockwise { player: 0 },
            ),
            (self.player1.hold.into(), GameInputKey::Hold { player: 0 }),
        ]);

        match self.player2 {
            None => {}
            Some(p2) => {
                result.insert(p2.move_left.into(), GameInputKey::MoveLeft { player: 1 });
                result.insert(p2.move_right.into(), GameInputKey::MoveRight { player: 1 });
                result.insert(p2.soft_drop.into(), GameInputKey::SoftDrop { player: 1 });
                result.insert(p2.hard_drop.into(), GameInputKey::HardDrop { player: 1 });
                result.insert(
                    p2.rotate_anticlockwise.into(),
                    GameInputKey::RotateAnticlockwise { player: 1 },
                );
                result.insert(
                    p2.rotate_clockwise.into(),
                    GameInputKey::RotateClockwise { player: 1 },
                );
                result.insert(p2.hold.into(), GameInputKey::Hold { player: 1 });
            }
        }

        result
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    pub music_volume: f64,
    pub effects_volume: f64,
}

impl AudioConfig {
    pub fn music_volume(&self) -> i32 {
        (self.music_volume * MAX_VOLUME as f64).round() as i32
    }

    pub fn effects_volume(&self) -> i32 {
        (self.effects_volume * MAX_VOLUME as f64).round() as i32
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct VideoConfig {
    pub mode: VideoMode,
    pub vsync: bool,
    pub disable_screensaver: bool,
    pub integer_scale: bool
}

impl VideoConfig {
    pub fn screen_padding_pct(&self) -> f64 {
        if self.integer_scale {
            // need a bigger buffer on the modern theme to line it up when integer scaling the retro themes
            0.05
        } else {
            0.02
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            video: VideoConfig {
                #[cfg(not(feature = "retro_handheld"))]
                mode: VideoMode::Window {
                    width: 1280,
                    height: 720,
                },
                #[cfg(feature = "retro_handheld")]
                mode: VideoMode::FullScreen {
                    width: 640,
                    height: 480,
                },
                vsync: true,
                disable_screensaver: true,

                // disable integer scaling to better fill small retro handheld screen
                // otherwise keep it enabled as it does look better
                integer_scale: !cfg!(feature = "retro_handheld")
            },
            audio: AudioConfig {
                music_volume: 0.5,
                effects_volume: 1.0,
            },
            /*
              ArkOS Default Controls:
              A= Keycode::X
              B= Keycode::Z
              X= Keycode::C
              Y= Keycode::A
              L1= Keycode::RShift
              L2= Keycode::Home
              R1= Keycode::LShift
              R2= Keycode::End
              Select= Keycode::Esc
              Start= Keycode::Return
            */
            input: InputConfig {
                menu: MenuInputConfig {
                    up: Keycode::UP.into(),
                    down: Keycode::DOWN.into(),
                    left: Keycode::LEFT.into(),
                    right: Keycode::RIGHT.into(),
                    select: Keycode::X.into(),
                    start: Keycode::RETURN.into(),
                },
                player1: GameInputConfig {
                    move_left: Keycode::LEFT.into(),
                    move_right: Keycode::RIGHT.into(),
                    soft_drop: Keycode::DOWN.into(),
                    hard_drop: Keycode::UP.into(),
                    rotate_clockwise: Keycode::X.into(),
                    rotate_anticlockwise: Keycode::Z.into(),
                    hold: Keycode::LSHIFT.into(),
                },
                player2: None,
                #[cfg(feature = "retro_handheld")] pause: Keycode::RETURN.into(),
                #[cfg(not(feature = "retro_handheld"))] pause: Keycode::F1.into(),
                #[cfg(feature = "retro_handheld")] next_theme: Keycode::RSHIFT.into(),
                #[cfg(not(feature = "retro_handheld"))] next_theme: Keycode::F2.into(),
                quit: Keycode::ESCAPE.into(),
            },
        }
    }
}

#[cfg(feature = "retro_handheld")]
pub fn config_path(name: &str) -> Result<PathBuf, String> {
    let mut absolute = std::env::current_dir().map_err(|e| e.to_string())?;
    absolute.push(format!("{}.yml", name));
    Ok(absolute)
}

#[cfg(not(feature = "retro_handheld"))]
pub fn config_path(name: &str) -> Result<PathBuf, String> {
    confy::get_configuration_file_path(crate::build_info::PKG_NAME, name)
        .map_err(|e| e.to_string())
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = config_path("config")?;

        #[cfg(debug_assertions)]
        println!("loading config: {}", config_path.to_str().unwrap());

        match confy::load_path(&config_path) {
            Ok(config) => Ok(config),
            Err(ConfyError::BadYamlData(error)) => {
                println!("Bad config file at {}, {}, loading defaults", config_path.to_str().unwrap(), error);
                Ok(Self::default())
            }
            Err(error) => Err(format!("{}", error)),
        }
    }
}


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GameKey(i32);

impl Into<Keycode> for GameKey {
    fn into(self) -> Keycode {
        Keycode::from_i32(self.0).unwrap()
    }
}

impl From<Keycode> for GameKey {
    fn from(value: Keycode) -> Self {
        GameKey(value.into())
    }
}

impl Serialize for GameKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let keycode: Keycode = (*self).into();
        serializer.serialize_str(&keycode.name())
    }
}

impl<'de> Deserialize<'de> for GameKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        struct GameKeyVisitor;
        impl<'de> Visitor<'de> for GameKeyVisitor {
            type Value = GameKey;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a string representation of an SDL2 KeyCode")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match Keycode::from_name(v) {
                    None => Err(E::custom(format!("invalid keycode '{}'", v))),
                    Some(keycode) => Ok(keycode.into())
                }
            }
        }
        deserializer.deserialize_string(GameKeyVisitor)
    }
}
