use std::collections::HashSet;
use crate::scale::Scale;
use crate::theme::all::AllThemes;
use crate::theme::Theme;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};

use sdl2::video::WindowContext;
use std::time::Duration;
use crate::animate::PlayerAnimations;
use crate::game::event::ColoredBlock;
use crate::game::GameSpeed;
use crate::game::geometry::BottlePoint;
use crate::game::pill::{PillShape, Vitamins};
use crate::game::rules::{GameConfig, MatchThemes};
use crate::player::MatchState;
use crate::theme::scene::SceneRender;

const THEME_FADE_DURATION: Duration = Duration::from_millis(1000);

pub struct PlayerTextures<'a> {
    pub background: Texture<'a>,
    pub bottle: Texture<'a>,
}

impl<'a> PlayerTextures<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        background_size: (u32, u32),
        bottle_size: (u32, u32),
    ) -> Result<Self, String> {
        let (bg_width, bg_height) = background_size;
        let mut background = texture_creator
            .create_texture_target(None, bg_width, bg_height)
            .map_err(|e| e.to_string())?;
        background.set_blend_mode(BlendMode::Blend);

        let (bottle_width, bottle_height) = bottle_size;
        let mut bottle = texture_creator
            .create_texture_target(None, bottle_width, bottle_height)
            .map_err(|e| e.to_string())?;
        bottle.set_blend_mode(BlendMode::Blend);

        Ok(Self { background, bottle })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureMode {
    Background(u32),
    Bottle(u32),
}

#[derive(Clone, Debug)]
struct ThemedPlayer {
    player: u32,
    bg_snip: Rect,
    bottle_snip: Rect,
    animations: PlayerAnimations
}

impl ThemedPlayer {
    pub fn new(player: u32, theme: &Theme, scale: Scale) -> Self {
        let (theme_width, theme_height) = theme.background_size();
        let mut bg_snip = scale.scale_rect(Rect::new(0, 0, theme_width, theme_height));
        bg_snip.center_on(scale.player_window(player).center());
        let bottle_snip = scale.scale_and_offset_rect(theme.bottle_snip(), bg_snip.x(), bg_snip.y());
        let animations = PlayerAnimations::new(theme);
        Self {
            player,
            bg_snip,
            bottle_snip,
            animations
        }
    }

    pub fn update_animations(&mut self, delta: Duration) {
        self.animations.update(delta);
    }
}

pub struct ScaledTheme<'a> {
    theme: &'a Theme<'a>,
    bg_source_snip: Rect,
    bottle_source_snip: Rect,
    player_themes: Vec<ThemedPlayer>,
    scale: Scale
}

impl<'a> ScaledTheme<'a> {
    fn new(theme: &'a Theme, players: u32, window_size: (u32, u32)) -> Self {
        let scale = Scale::new(
            players,
            theme.background_size(),
            window_size,
            theme.geometry().block_size(),
        );
        let (theme_width, theme_height) = theme.background_size();
        let bg_source_snip = Rect::new(0, 0, theme_width, theme_height);
        let bottle_rect = theme.bottle_snip();
        let bottle_source_snip = Rect::new(0, 0, bottle_rect.width(), bottle_rect.height());
        let player_themes = (0..players)
            .map(|pid| ThemedPlayer::new(pid, theme, scale))
            .collect::<Vec<ThemedPlayer>>();
        Self {
            theme,
            bg_source_snip,
            bottle_source_snip,
            player_themes,
            scale,
        }
    }

    pub fn rows_to_pixels(&self, value: u32) -> u32 {
        let raw_pixels = self.theme.geometry().block_size() * value;
        self.scale.scale_length(raw_pixels)
    }

    pub fn update(&mut self, delta: Duration) {
        for player in self.player_themes.iter_mut() {
            player.update_animations(delta);
        }
    }

    pub fn animations_mut(&mut self, player: u32) -> &mut PlayerAnimations {
        &mut self.player_themes.get_mut(player as usize).unwrap().animations
    }

    pub fn is_pause_required_for_animation(&self, player: u32) -> bool {
        self.player_themes[player as usize].animations.is_animating()
    }

    pub fn is_animating_next_level_interstitial(&self) -> bool {
        for player in self.player_themes.iter() {
            if player.animations.next_level_interstitial().state().is_some() {
                return true;
            }
        }
        return false;
    }
}

pub struct ThemeContext<'a> {
    current: usize,
    themes: Vec<ScaledTheme<'a>>,
    fade_buffer: Texture<'a>,
    fade_duration: Option<Duration>,
}

impl<'a> ThemeContext<'a> {
    pub fn new(
        all_themes: &'a AllThemes,
        texture_creator: &'a TextureCreator<WindowContext>,
        game_config: GameConfig,
        window_size: (u32, u32),
    ) -> Result<Self, String> {
        let (window_width, window_height) = window_size;

        let mut fade_buffer = texture_creator
            .create_texture_target(None, window_width, window_height)
            .map_err(|e| e.to_string())?;
        fade_buffer.set_blend_mode(BlendMode::Blend);

        let current = match game_config.themes() {
            MatchThemes::All | MatchThemes::Nes => 0,
            MatchThemes::Snes => 1,
            MatchThemes::N64 => 2,
        };

        Ok(Self {
            current,
            themes: all_themes
                .all()
                .iter()
                .map(|theme| ScaledTheme::new(theme, game_config.players(), window_size))
                .collect(),
            fade_buffer,
            fade_duration: None,
        })
    }

    pub fn max_background_size(&self) -> (u32, u32) {
        let sizes = self
            .themes
            .iter()
            .map(|theme| theme.theme.background_size());
        let width = sizes.clone().map(|(w, _)| w).max().unwrap();
        let height = sizes.clone().map(|(_, h)| h).max().unwrap();
        (width, height)
    }

    pub fn max_bottle_size(&self) -> (u32, u32) {
        let rects = self.themes.iter().map(|theme| theme.theme.bottle_snip());
        let width = rects.clone().map(|r| r.width()).max().unwrap();
        let height = rects.clone().map(|r| r.height()).max().unwrap();
        (width, height)
    }

    pub fn theme(&self) -> &Theme<'a> {
        self.themes[self.current].theme
    }

    pub fn player_bottle_snip(&self, player: u32) -> Rect {
        let theme = &self.themes[self.current];
        theme
            .player_themes
            .get(player as usize)
            .unwrap()
            .bottle_snip
    }

    pub fn player_animations(&self, player: u32) -> &PlayerAnimations {
        &self.current().player_themes[player as usize].animations
    }

    pub fn current(&self) -> &ScaledTheme {
        &self.themes[self.current]
    }

    pub fn update(&mut self, delta: Duration) {
        for theme in self.themes.iter_mut() {
            theme.update(delta);
        }
    }

    pub fn animate_destroy(&mut self, player: u32, blocks: Vec<ColoredBlock>) {
        for theme in self.themes.iter_mut() {
            theme.animations_mut(player).destroy_mut().add(blocks.clone());
        }
    }

    pub fn animate_impact(&mut self, player: u32) {
        for theme in self.themes.iter_mut() {
            theme.animations_mut(player).impact_mut().impact();
        }
    }

    pub fn animate_lock(&mut self, player: u32, vitamins: Vitamins) {
        for theme in self.themes.iter_mut() {
            theme.animations_mut(player).lock_mut().lock(vitamins);
        }
    }

    pub fn animate_hard_drop(&mut self, player: u32, vitamins: Vitamins, dropped_rows: u32) {
        for theme in self.themes.iter_mut() {
            theme.animations_mut(player).hard_drop_mut().hard_drop(vitamins, dropped_rows);
        }
    }

    pub fn animate_spawn(&mut self, player: u32, shape: PillShape, is_hold: bool) {
        for theme in self.themes.iter_mut() {
            theme.animations_mut(player).spawn_mut().spawn(shape, is_hold);
        }
    }

    pub fn animate_game_over(&mut self, player: u32) {
        for theme in self.themes.iter_mut() {
            theme.animations_mut(player).game_over_mut().game_over();
        }
    }

    pub fn animate_victory(&mut self, player: u32) {
        for theme in self.themes.iter_mut() {
            theme.animations_mut(player).victory_mut().victory();
        }
    }

    pub fn animate_next_level_interstitial(&mut self, player: u32) {
        for theme in self.themes.iter_mut() {
            theme.animations_mut(player).next_level_interstitial_mut().display();
        }
    }

    pub fn animate_next_level(&mut self, player: u32, viruses: &[ColoredBlock]) {
        for theme in self.themes.iter_mut() {
            theme.animations_mut(player).next_level_mut().next_level(viruses);
        }
    }

    pub fn maybe_dismiss_next_level_interstitial(&mut self, player: u32) -> bool {
        let mut result = false;
        for index in 0..self.themes.len() {
            let theme_result = self.themes[index].animations_mut(player).next_level_interstitial_mut().dismiss();
            if index == self.current {
                result = theme_result;
            }
        }
        result
    }

    pub fn is_animating_next_level_interstitial(&self) -> bool {
        self.themes[self.current].is_animating_next_level_interstitial()
    }

    pub fn maybe_dismiss_game_over(&mut self) {
        for theme in self.themes.iter_mut() {
            for player in theme.player_themes.iter_mut() {
                player.animations.game_over_mut().dismiss();
                player.animations.victory_mut().dismiss();
            }
        }
    }

    pub fn is_any_game_over_dismissed(&self) -> bool {
        for player in self.themes[self.current].player_themes.iter() {
            if player.animations.game_over().state().map(|s| s.is_dismissed()).unwrap_or(false) {
                return true;
            }
        }
        false
    }

    pub fn is_all_post_game_animation_complete(&self) -> bool {
        for player in self.themes[self.current].player_themes.iter() {
            if let Some(game_over) = player.animations.game_over().state() {
                if !game_over.is_complete() {
                    return false;
                }
            }

            if let Some(victory) = player.animations.victory().state() {
                if !victory.is_complete() {
                    return false;
                }
            }
        }
        true
    }

    pub fn fade_into_next_theme(&mut self, canvas: &mut WindowCanvas, match_state: MatchState, is_single_player: bool) -> Result<(), String> {
        for theme in self.themes.iter_mut() {
            for player in theme.player_themes.iter_mut() {
                player.animations.reset();
            }
        }
        self.current = (self.current + 1) % self.themes.len();

        self.start_fade(canvas)?;

        // handle music
        let audio = self.theme().audio();
        match match_state {
            MatchState::Normal if self.is_animating_next_level_interstitial() => audio.play_next_level_music()?,
            MatchState::Normal => audio.fade_in_game_music()?,
            MatchState::Paused => {
                audio.play_game_music()?;
                audio.pause_music();
            }
            MatchState::GameOver { .. } => {
                if is_single_player {
                    audio.play_game_over_music()?
                } else {
                    audio.play_victory_music()?
                }
            }
        }

        Ok(())
    }

    fn start_fade(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        self.fade_duration = Some(Duration::ZERO);

        let query = self.fade_buffer.query();
        let pixels = canvas.read_pixels(None, query.format)?;
        self.fade_buffer
            .update(
                None,
                pixels.as_slice(),
                query.format.byte_size_per_pixel() * query.width as usize,
            )
            .map_err(|e| e.to_string())
    }

    pub fn is_fading(&self) -> bool {
        self.fade_duration.is_some()
    }

    pub fn draw_scene(&self, canvas: &mut WindowCanvas, speed: GameSpeed) -> Result<(), String> {
        let current = self.current();
        current.theme.scene(speed).draw(canvas, &current.scale)
    }

    pub fn draw_players(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_refs: &mut [(&mut Texture, TextureMode)],
        delta: Duration
    ) -> Result<(), String> {
        let current = self.current();
        for (texture, texture_mode) in texture_refs.iter_mut() {
            match texture_mode {
                TextureMode::Background(pid) => {
                    let player = &current.player_themes[*pid as usize];
                    canvas.copy(texture, current.bg_source_snip, player.bg_snip)?;
                }
                TextureMode::Bottle(pid) => {
                    let player = &current.player_themes[*pid as usize];
                    let (offset_x, offset_y) = player.animations.impact().current_offset();
                    let dst = current.scale.offset_proportional_to_block_size(
                        player.bottle_snip,
                        offset_x,
                        offset_y,
                    );
                    canvas.copy(texture, current.bottle_source_snip, dst)?;
                }
            }
        }

        // check if we should be fading out the previous theme
        match self.fade_duration {
            None => {}
            Some(duration) => {
                let duration = duration + delta;
                if duration > THEME_FADE_DURATION {
                    self.fade_duration = None;
                } else {
                    let alpha = 255.0 * duration.as_millis() as f64
                        / THEME_FADE_DURATION.as_millis() as f64;
                    self.fade_buffer.set_alpha_mod(255 - alpha as u8);
                    canvas.copy(&self.fade_buffer, None, None)?;
                    self.fade_duration = Some(duration);
                }
            }
        }

        Ok(())
    }
}
