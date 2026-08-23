use crate::scale::Scale;
use crate::theme::all::AllThemes;
use crate::theme::{Theme, ThemeName};
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};

use crate::animate::event::AnimationEvent;
use crate::animate::PlayerAnimations;
use crate::game::event::ColoredBlock;
use crate::game::geometry::BottlePoint;
use crate::game::pill::{PillShape, Vitamins};
use crate::game::rules::{GameConfig, MatchThemes};
use crate::game::GameSpeed;
use crate::particles::render::ParticleRender;
use crate::player::{Match, MatchState};
use crate::theme::sound::AudioTheme;

use sdl2::video::WindowContext;
use std::time::Duration;
use sdl2::pixels::PixelFormatEnum::RGBA8888;
use crate::config::VideoConfig;

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
            .create_texture_target(RGBA8888, bg_width, bg_height)
            .map_err(|e| e.to_string())?;
        background.set_blend_mode(BlendMode::Blend);

        let (bottle_width, bottle_height) = bottle_size;
        let mut bottle = texture_creator
            .create_texture_target(RGBA8888, bottle_width, bottle_height)
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
    bg_snip: Rect,
    bottle_snip: Rect,
    game_snip: Rect,
    animations: PlayerAnimations,
}

impl ThemedPlayer {
    pub fn new(player: u32, theme: &Theme, scale: Scale) -> Self {
        let (theme_width, theme_height) = theme.background_size();
        let mut bg_snip = scale.scale_rect(Rect::new(0, 0, theme_width, theme_height));
        bg_snip.center_on(scale.player_window(player).center());
        let bottle_snip =
            scale.scale_and_offset_rect(theme.bottle_snip(), bg_snip.x(), bg_snip.y());
        let game_snip =
            scale.scale_and_offset_rect(theme.geometry().game_snip(), bg_snip.x(), bg_snip.y());
        let animations = PlayerAnimations::new(player, theme);
        Self {
            bg_snip,
            bottle_snip,
            game_snip,
            animations,
        }
    }

    pub fn update_animations(&mut self, delta: Duration) -> Vec<AnimationEvent> {
        self.animations.update(delta)
    }
}

pub struct ScaledTheme<'a> {
    theme: &'a Theme<'a>,
    bg_source_snip: Rect,
    bottle_source_snip: Rect,
    player_themes: Vec<ThemedPlayer>,
    scale: Scale,
}

impl<'a> ScaledTheme<'a> {
    fn new(theme: &'a Theme, players: u32, window_size: (u32, u32), video_config: VideoConfig) -> Self {
        let scale = Scale::new(
            players,
            theme.background_size(),
            window_size,
            theme.geometry().block_size(),
            video_config,
            // the modern theme does its own scaling
            theme.name() == ThemeName::Particle,
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

    pub fn update_animations(&mut self, delta: Duration) -> Vec<AnimationEvent> {
        self.player_themes
            .iter_mut()
            .flat_map(|p| p.update_animations(delta))
            .collect()
    }

    pub fn animations_mut(&mut self, player: u32) -> &mut PlayerAnimations {
        &mut self
            .player_themes
            .get_mut(player as usize)
            .unwrap()
            .animations
    }

    pub fn is_pause_required_for_animation(&self, player: u32) -> bool {
        self.player_themes[player as usize]
            .animations
            .is_animating()
    }

}

pub struct ThemeContext<'a> {
    /// the current theme index of each player
    current: Vec<usize>,
    themes: Vec<ScaledTheme<'a>>,
    fade_buffer: Texture<'a>,
    /// per-player theme fade timer
    fades: Vec<Option<Duration>>,
    /// the player whose theme music is playing
    music_player: u32,
    /// the theme index whose music is playing
    music_theme: Option<usize>,
}

impl<'a> ThemeContext<'a> {
    pub fn new(
        all_themes: &'a AllThemes,
        texture_creator: &'a TextureCreator<WindowContext>,
        game_config: GameConfig,
        window_size: (u32, u32),
        video_config: VideoConfig
    ) -> Result<Self, String> {
        let (window_width, window_height) = window_size;

        let mut fade_buffer = texture_creator
            .create_texture_target(RGBA8888, window_width, window_height)
            .map_err(|e| e.to_string())?;
        fade_buffer.set_blend_mode(BlendMode::Blend);

        let initial = match game_config.themes() {
            MatchThemes::All | MatchThemes::Nes => 0,
            MatchThemes::Snes => 1,
            MatchThemes::N64 => 2,
            MatchThemes::Particle => 3,
        };
        let players = game_config.players() as usize;

        Ok(Self {
            current: vec![initial; players],
            themes: all_themes
                .all()
                .iter()
                .map(|theme| ScaledTheme::new(theme, game_config.players(), window_size, video_config))
                .collect(),
            fade_buffer,
            fades: vec![None; players],
            music_player: 0,
            music_theme: None,
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

    fn players(&self) -> u32 {
        self.current.len() as u32
    }

    /// the theme a player is currently on
    pub fn theme(&self, player: u32) -> &Theme<'a> {
        self.themes[self.current[player as usize]].theme
    }

    pub fn current(&self, player: u32) -> &ScaledTheme<'a> {
        &self.themes[self.current[player as usize]]
    }

    /// the audio of the theme whose music is playing: the theme of the winning player
    pub fn music_audio(&self) -> &AudioTheme {
        let index = self
            .music_theme
            .unwrap_or(self.current[self.music_player as usize]);
        self.themes[index].theme.audio()
    }

    pub fn player_bottle_snip(&self, player: u32) -> Rect {
        self.current(player).player_themes[player as usize].game_snip
    }

    pub fn player_animations(&self, player: u32) -> &PlayerAnimations {
        &self.current(player).player_themes[player as usize].animations
    }

    pub fn is_pause_required_for_animation(&self, player: u32) -> bool {
        self.current(player).is_pause_required_for_animation(player)
    }

    pub fn update_animations(&mut self, delta: Duration) -> Vec<AnimationEvent> {
        let mut events = vec![];
        for (id, theme) in self.themes.iter_mut().enumerate() {
            for event in theme.update_animations(delta).into_iter() {
                // only emit from the theme the player is currently on
                let AnimationEvent::Finished { player, .. } = event;
                if self.current[player as usize] == id {
                    events.push(event);
                }
            }
        }
        events
    }

    pub fn animate_destroy(&mut self, player: u32, blocks: Vec<ColoredBlock>) {
        for theme in self.themes.iter_mut() {
            theme
                .animations_mut(player)
                .destroy_mut()
                .add(blocks.clone());
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
            theme
                .animations_mut(player)
                .hard_drop_mut()
                .hard_drop(vitamins, dropped_rows);
        }
    }

    pub fn animate_spawn(&mut self, player: u32, shape: PillShape, is_hold: bool) {
        for theme in self.themes.iter_mut() {
            theme
                .animations_mut(player)
                .throw_mut()
                .throw(shape, is_hold);
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
            theme
                .animations_mut(player)
                .next_level_interstitial_mut()
                .display();
        }
    }

    pub fn animate_next_level(&mut self, player: u32, viruses: &[ColoredBlock]) {
        for theme in self.themes.iter_mut() {
            theme
                .animations_mut(player)
                .next_level_mut()
                .next_level(viruses);
        }
    }

    pub fn maybe_dismiss_next_level_interstitial(&mut self, player: u32) -> bool {
        let mut result = false;
        for index in 0..self.themes.len() {
            let theme_result = self.themes[index]
                .animations_mut(player)
                .next_level_interstitial_mut()
                .dismiss();
            if index == self.current[player as usize] {
                result = theme_result;
            }
        }
        result
    }

    pub fn is_animating_next_level_interstitial(&self) -> bool {
        (0..self.players()).any(|player| {
            self.player_animations(player)
                .next_level_interstitial()
                .state()
                .is_some()
        })
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
        (0..self.players()).any(|player| {
            self.player_animations(player)
                .game_over()
                .state()
                .map(|s| s.is_dismissed())
                .unwrap_or(false)
        })
    }

    pub fn is_all_post_game_animation_complete(&self) -> bool {
        for player in 0..self.players() {
            let animations = self.player_animations(player);
            if let Some(game_over) = animations.game_over().state() {
                if !game_over.is_complete() {
                    return false;
                }
            }

            if let Some(victory) = animations.victory().state() {
                if !victory.is_complete() {
                    return false;
                }
            }
        }
        true
    }

    /// advance a single player to their next theme, cross-fading only their side of the screen
    pub fn fade_into_next_theme(
        &mut self,
        player: u32,
        canvas: &mut WindowCanvas,
    ) -> Result<(), String> {
        for theme in self.themes.iter_mut() {
            theme.animations_mut(player).reset();
        }
        let index = player as usize;
        self.current[index] = (self.current[index] + 1) % self.themes.len();
        self.start_fade(player, canvas)
    }

    pub fn fade_all_into_next_theme(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        for player in 0..self.players() {
            self.fade_into_next_theme(player, canvas)?;
        }
        Ok(())
    }

    /// keep the music on the theme of the winning player. the leader is only re-evaluated when
    /// `reevaluate_leader` is set (between stages), otherwise only the theme itself is checked
    /// i.e. the music owner changed theme. returns true if the music was (re)started.
    pub fn sync_music(
        &mut self,
        fixture: &Match,
        reevaluate_leader: bool,
        is_single_player: bool,
    ) -> Result<bool, String> {
        if reevaluate_leader {
            if let Some(leader) = fixture.leading_player() {
                self.music_player = leader;
            }
        }
        let wanted = self.current[self.music_player as usize];
        if self.music_theme == Some(wanted) {
            return Ok(false);
        }
        self.music_theme = Some(wanted);

        let audio = self.themes[wanted].theme.audio();
        match fixture.state() {
            // Only single player uses next-level *music*; in multiplayer the stage clear is a
            // jingle and game music must keep playing, otherwise another player's still-open
            // interstitial would swap in a play-once track and leave the match silent.
            MatchState::Normal if is_single_player && self.is_animating_next_level_interstitial() => {
                audio.play_next_level_music()?
            }
            MatchState::Normal => audio.fade_in_game_music()?,
            MatchState::Paused => {
                audio.play_game_music()?;
                audio.pause_music()?
            }
            MatchState::GameOver { .. } => {
                if is_single_player {
                    audio.play_game_over_music()?
                } else {
                    audio.play_victory_music()?
                }
            }
        }
        Ok(true)
    }

    fn player_clip(&self, player: u32) -> Rect {
        self.current(player).scale.player_clip(player)
    }

    fn start_fade(&mut self, player: u32, canvas: &mut WindowCanvas) -> Result<(), String> {
        self.fades[player as usize] = Some(Duration::ZERO);

        // only snapshot this player's side so another player's in-progress fade is untouched
        let clip = self.player_clip(player);
        let query = self.fade_buffer.query();
        let pixels = canvas.read_pixels(clip, query.format)?;
        self.fade_buffer
            .update(
                clip,
                pixels.as_slice(),
                query.format.byte_size_per_pixel() * clip.width() as usize,
            )
            .map_err(|e| e.to_string())
    }

    pub fn is_fading(&self, player: u32) -> bool {
        self.fades[player as usize].is_some()
    }

    /// draw each player's scene backdrop as if it filled the whole window, clipped to their side
    pub fn draw_scene(&self, canvas: &mut WindowCanvas, speed: GameSpeed) -> Result<(), String> {
        for player in 0..self.players() {
            let current = self.current(player);
            canvas.set_clip_rect(self.player_clip(player));
            current.theme.scene(speed).draw(canvas, &current.scale)?;
        }
        canvas.set_clip_rect(None);
        Ok(())
    }

    /// draw the background particles, clipped to the sides of the players on a particle scene
    pub fn draw_scene_particles(
        &self,
        canvas: &mut WindowCanvas,
        particles: &mut ParticleRender,
    ) -> Result<(), String> {
        for player in 0..self.players() {
            if self.player_renders_scene_particles(player) {
                canvas.set_clip_rect(self.player_clip(player));
                particles.draw(canvas)?;
            }
        }
        canvas.set_clip_rect(None);
        Ok(())
    }

    pub fn draw_players(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_refs: &mut [(&mut Texture, TextureMode)],
        delta: Duration,
    ) -> Result<(), String> {
        for (texture, texture_mode) in texture_refs.iter_mut() {
            match texture_mode {
                TextureMode::Background(pid) => {
                    let current = self.current(*pid);
                    let player = &current.player_themes[*pid as usize];
                    canvas.copy(texture, current.bg_source_snip, player.bg_snip)?;
                }
                TextureMode::Bottle(pid) => {
                    let current = self.current(*pid);
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

        // fade out the previous theme on each side that is changing
        for player in 0..self.players() {
            let Some(duration) = self.fades[player as usize] else {
                continue;
            };
            let duration = duration + delta;
            if duration > THEME_FADE_DURATION {
                self.fades[player as usize] = None;
            } else {
                let alpha = 255.0 * duration.as_millis() as f64
                    / THEME_FADE_DURATION.as_millis() as f64;
                self.fade_buffer.set_alpha_mod(255 - alpha as u8);
                let clip = self.player_clip(player);
                canvas.copy(&self.fade_buffer, clip, clip)?;
                self.fades[player as usize] = Some(duration);
            }
        }

        Ok(())
    }

    pub fn player_block_snips(&self, player: u32, points: Vec<BottlePoint>) -> Vec<Rect> {
        let theme = self.current(player);
        let player = &theme.player_themes[player as usize];
        let geometry = theme.theme.geometry();
        points
            .into_iter()
            .map(|p| geometry.raw_block(p))
            .map(|r| {
                theme
                    .scale
                    .scale_and_offset_rect(r, player.bottle_snip.x(), player.bottle_snip.y())
            })
            .collect()
    }

    pub fn player_block_snips_masked(
        &self,
        player: u32,
        blocks: Vec<ColoredBlock>,
        lattice_spacing: u32,
    ) -> Vec<Point> {
        let theme = self.current(player);
        let player = &theme.player_themes[player as usize];
        let geometry = theme.theme.geometry();
        let sprites = theme.theme.sprites();

        blocks
            .into_iter()
            .flat_map(|b| {
                if b.is_virus {
                    sprites.virus_mask(b.color)
                } else {
                    sprites.garbage_mask()
                }
                .lattice(geometry.point(b.position), lattice_spacing)
            })
            .map(|p| {
                theme.scale.scale_and_offset_point(
                    p,
                    player.bottle_snip.x(),
                    player.bottle_snip.y(),
                )
            })
            .collect()
    }

    pub fn player_vitamin_snips(&self, player: u32, vitamins: Vitamins) -> [Rect; 2] {
        let theme = self.current(player);
        let player = &theme.player_themes[player as usize];
        let geometry = theme.theme.geometry();
        vitamins.map(|v| geometry.raw_block(v.position())).map(|r| {
            theme
                .scale
                .scale_and_offset_rect(r, player.bottle_snip.x(), player.bottle_snip.y())
        })
    }

    pub fn player_renders_scene_particles(&self, player: u32) -> bool {
        self.theme(player).scene(GameSpeed::Low).is_particles()
    }

    /// true if any player is on a theme with a particle scene
    pub fn render_scene_particles(&self) -> bool {
        (0..self.players()).any(|player| self.player_renders_scene_particles(player))
    }
}
