//! The application shell: window, audio, menus and the match loop. Generic over the game
//! being played; a launcher decides which games and themes to run.

use crate::animate::event::{AnimationEvent, AnimationType};
use crate::audio;
use crate::config::{Config, VideoMode};
use crate::frame_rate::FrameRate;
use crate::game::{Game, GameEvent, StageTransition};
use crate::game_input::{GameInputContext, GameInputKey};
use crate::high_score::event::HighScoreEntryEvent;
use crate::high_score::render::HighScoreRender;
use crate::high_score::table::HighScoreTable;
use crate::high_score::NewHighScore;
use crate::icon::app_icon;
use crate::menu::sound::MenuSound;
use crate::menu::{Menu, MenuItem};
use crate::menu_input::{MenuInputContext, MenuInputKey};
use crate::particles::prescribed::{
    prescribed_fireworks, prescribed_orbit, prescribed_piece_race, PlayerTargetedParticles,
    RaceTheme,
};
use crate::particles::render::ParticleRender;
use crate::particles::scale::Scale as ParticleScale;
use crate::particles::source::ParticleSource;
use crate::particles::Particles;
use crate::render::context::{PlayerTextures, PlayerThemes, TextureMode, ThemeContext};
use crate::render::pause::PausedScreen;
use crate::render::{GameRender, Theme};
use crate::session::{Match, MatchRules, MatchState};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use sdl2::{EventPump, Sdl};
use std::ops::Range;

/// Simultaneous sound effects per player (SDL_mixer's default channel count).
const EFFECT_CHANNELS_PER_PLAYER: u32 = 8;

pub const MAX_PARTICLES_PER_PLAYER: usize = 100000;
pub const MAX_BACKGROUND_PARTICLES: usize = 100000;

/// How a menu loop ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuExit<R> {
    Start,
    Back,
    Quit,
    Custom(R),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuMusic {
    Title,
    Menu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostGameAction {
    NewHighScore(NewHighScore),
    ReturnToMenu,
    Quit,
}

/// Which theme a player plays on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    /// every theme in order, advancing at each stage
    All,
    /// one theme, by index within the player's set
    Fixed(usize),
}

impl ThemeMode {
    pub fn initial(&self) -> usize {
        match self {
            ThemeMode::All => 0,
            ThemeMode::Fixed(index) => *index,
        }
    }
}

/// One player's game setup within a match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerSettings {
    /// the player's themes: a range of indices into the match's theme list
    pub themes: Range<usize>,
    pub theme_mode: ThemeMode,
}

impl PlayerSettings {
    fn player_themes(&self) -> PlayerThemes {
        PlayerThemes::new(
            self.themes.clone(),
            self.themes.start + self.theme_mode.initial(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchSettings {
    pub rules: MatchRules,
    pub players: Vec<PlayerSettings>,
    /// which high score table the match competes for
    pub high_score_key: String,
}

pub struct App {
    config: Config,
    _sdl: Sdl,
    canvas: WindowCanvas,
    event_pump: EventPump,
    _audio: audio::Audio,
    menu_sound: MenuSound,
    particle_scale: ParticleScale,
    max_players: u32,
}

impl App {
    pub fn new(max_players: u32, icon: &[u8]) -> Result<Self, String> {
        let config = Config::load()?;
        let sdl = sdl2::init()?;
        let video = sdl.video()?;

        // let resolutions: BTreeSet<(i32, i32)> = (0..video.num_display_modes(0)?)
        //     .into_iter()
        //     .map(|i| video.display_mode(0, i).unwrap())
        //     .map(|mode| (mode.w, mode.h))
        //     .collect();

        if config.video.disable_screensaver && video.is_screen_saver_enabled() {
            video.disable_screen_saver();
        }

        let (width, height) = match config.video.mode {
            VideoMode::Window { width, height } => (width, height),
            VideoMode::FullScreen { width, height } => (width, height),
            _ => (1, 1),
        };

        let mut window_builder = video.window(crate::app_info::get().name, width, height);
        match config.video.mode {
            VideoMode::FullScreen { .. } => {
                window_builder.fullscreen();
            }
            VideoMode::FullScreenDesktop => {
                window_builder.fullscreen_desktop();
            }
            _ => {}
        };

        let mut window = window_builder
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        window.set_icon(app_icon(icon)?);

        let canvas_builder = window.into_canvas().target_texture().accelerated();

        let canvas = if config.video.vsync {
            canvas_builder.present_vsync()
        } else {
            canvas_builder
        }
        .build()
        .map_err(|e| e.to_string())?;

        let event_pump = sdl.event_pump()?;

        let audio = audio::Audio::open(
            &sdl.audio()?,
            max_players * EFFECT_CHANNELS_PER_PLAYER,
            config.audio.music_volume(),
        )?;
        let menu_sound = MenuSound::new(config.audio)?;

        Ok(Self {
            config,
            _sdl: sdl,
            canvas,
            event_pump,
            _audio: audio,
            menu_sound,
            particle_scale: ParticleScale::new((width, height)),
            max_players,
        })
    }

    pub fn config(&self) -> Config {
        self.config
    }

    pub fn canvas(&mut self) -> &mut WindowCanvas {
        &mut self.canvas
    }

    pub fn max_players(&self) -> u32 {
        self.max_players
    }

    pub fn particle_render<'a>(
        &mut self,
        texture_creator: &'a TextureCreator<WindowContext>,
        max_particles: usize,
        themes: Vec<&Theme<'a>>,
    ) -> Result<ParticleRender<'a>, String> {
        ParticleRender::new(
            &mut self.canvas,
            Particles::new(max_particles),
            texture_creator,
            self.particle_scale,
            themes,
        )
    }

    fn race_particle_source(&self, race: &[RaceTheme]) -> Box<dyn ParticleSource> {
        let (window_width, window_height) = self.canvas.window().size();
        prescribed_piece_race(
            Rect::new(0, 0, window_width, window_height),
            &self.particle_scale,
            race,
        )
    }

    fn fireworks_particle_source(&self) -> Box<dyn ParticleSource> {
        let (window_width, window_height) = self.canvas.window().size();
        prescribed_fireworks(
            Rect::new(0, 0, window_width, window_height),
            &self.particle_scale,
        )
    }

    fn orbit_particle_source(&self) -> Box<dyn ParticleSource> {
        let (window_width, window_height) = self.canvas.window().size();
        prescribed_orbit(
            Rect::new(0, 0, window_width, window_height),
            &self.particle_scale,
        )
    }

    /// Run a menu until the player starts, backs out or quits. `on_select` sees every change
    /// of a list item and every selected item by name and may end the menu with a value.
    pub fn run_menu<R>(
        &mut self,
        items: Vec<MenuItem>,
        title: String,
        subtitle: Option<String>,
        music: MenuMusic,
        particles: &mut ParticleRender,
        race: &[RaceTheme],
        mut on_select: impl FnMut(&str, &str) -> Option<MenuExit<R>>,
    ) -> Result<MenuExit<R>, String> {
        let texture_creator = self.canvas.texture_creator();
        let inputs = MenuInputContext::new(self.config.input);
        let mut menu = Menu::new(items, &mut self.canvas, &texture_creator, title, subtitle)?;

        particles.clear();
        particles.add_source(self.race_particle_source(race));

        let mut frame_rate = FrameRate::new();
        match music {
            MenuMusic::Title => self.menu_sound.play_title_music()?,
            MenuMusic::Menu => self.menu_sound.play_menu_music()?,
        }
        loop {
            let delta = frame_rate.update()?;
            for key in inputs.parse(self.event_pump.poll_iter()).into_iter() {
                if key == MenuInputKey::Quit {
                    return Ok(MenuExit::Quit);
                }

                match menu.read_key(key) {
                    None => match key {
                        MenuInputKey::Start => {
                            self.menu_sound.play_select()?;
                            return Ok(MenuExit::Start);
                        }
                        MenuInputKey::Back => return Ok(MenuExit::Back),
                        _ => {}
                    },
                    Some((name, action)) => {
                        if let Some(exit) = on_select(name, action) {
                            if !matches!(exit, MenuExit::Back) {
                                self.menu_sound.play_select()?;
                            }
                            return Ok(exit);
                        }
                    }
                }
                self.menu_sound.play_chime()?;
            }

            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();

            // particles
            particles.update(delta);
            particles.draw(&mut self.canvas)?;

            // menu
            menu.draw(&mut self.canvas)?;

            self.canvas.present();
        }
    }

    pub fn view_high_score(
        &mut self,
        key: &str,
        particles: &mut ParticleRender,
    ) -> Result<(), String> {
        let texture_creator = self.canvas.texture_creator();
        let inputs = MenuInputContext::new(self.config.input);
        let high_scores = HighScoreTable::load(key)?;
        if high_scores.entries().is_empty() {
            return Ok(());
        }

        let mut view = HighScoreRender::new(
            high_scores,
            &texture_creator,
            self.canvas.window().size(),
            None,
        )?;

        particles.clear();
        particles.add_source(self.fireworks_particle_source());

        let mut frame_rate = FrameRate::new();
        self.menu_sound.play_high_score_music()?;
        'menu: loop {
            let delta = frame_rate.update()?;
            let events = inputs.parse(self.event_pump.poll_iter());
            if !events.is_empty() {
                // any button press
                break 'menu;
            }
            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();

            // particles
            particles.update(delta);
            particles.draw(&mut self.canvas)?;

            view.draw(&mut self.canvas)?;

            self.canvas.present();
        }
        Ok(())
    }

    pub fn new_high_score(
        &mut self,
        key: &str,
        new_high_score: NewHighScore,
        particles: &mut ParticleRender,
    ) -> Result<(), String> {
        let texture_creator = self.canvas.texture_creator();
        let inputs = MenuInputContext::new(self.config.input);
        let high_scores = HighScoreTable::load(key)?;
        if high_scores.entries().is_empty() {
            return Ok(());
        }

        let mut table = HighScoreRender::new(
            high_scores,
            &texture_creator,
            self.canvas.window().size(),
            Some(new_high_score),
        )?;

        particles.clear();
        particles.add_source(self.fireworks_particle_source());

        let mut frame_rate = FrameRate::new();
        self.menu_sound.play_high_score_music()?;
        'menu: loop {
            let delta = frame_rate.update()?;

            for key in inputs.parse(self.event_pump.poll_iter()) {
                let event = match key {
                    MenuInputKey::Up => table.up(),
                    MenuInputKey::Down => table.down(),
                    MenuInputKey::Left => table.left(),
                    MenuInputKey::Right => table.right(),
                    MenuInputKey::Start => break 'menu,
                    MenuInputKey::Back => return Ok(()),
                    _ => None,
                };
                if event.is_none() {
                    continue;
                }
                match event.unwrap() {
                    HighScoreEntryEvent::Finished => break 'menu,
                    _ => self.menu_sound.play_chime()?,
                }
            }

            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();

            // particles
            particles.update(delta);
            particles.draw(&mut self.canvas)?;

            table.draw(&mut self.canvas)?;

            self.canvas.present();
        }

        if let Some(new_entry) = table.new_entry() {
            let mut high_scores = HighScoreTable::load(key).unwrap();
            high_scores.add_high_score(new_entry);
            high_scores.save(key)
        } else {
            Ok(())
        }
    }

    /// Play one match to its end.
    pub fn run_match<G: Game + GameRender>(
        &mut self,
        all_themes: &[Theme],
        games: Vec<G>,
        settings: MatchSettings,
        fg_particles: &mut ParticleRender,
        bg_particles: &mut ParticleRender,
    ) -> Result<PostGameAction, String> {
        let texture_creator = self.canvas.texture_creator();
        let mut inputs = GameInputContext::new(self.config.input);
        let players = games.len() as u32;
        assert_eq!(settings.players.len(), games.len());
        let theme_counts = settings
            .players
            .iter()
            .map(|p| p.themes.len() as u32)
            .collect::<Vec<u32>>();
        let mut fixture = Match::new(games, settings.rules, &theme_counts, &settings.high_score_key);
        let is_single_player = fixture.is_single_player();
        let window_size = self.canvas.window().size();
        let mut themes = ThemeContext::new(
            all_themes,
            &texture_creator,
            settings.players.iter().map(|p| p.player_themes()).collect(),
            window_size,
            self.config.video,
        )?;
        let mut player_textures = (0..players)
            .map(|_| {
                PlayerTextures::new(
                    &texture_creator,
                    themes.max_background_size(),
                    themes.max_board_size(),
                )
                .unwrap()
            })
            .collect::<Vec<PlayerTextures>>();

        // push mut refs of all textures and their render modes into a single vector so we can render to texture in one loop
        let mut texture_refs: Vec<(&mut Texture, TextureMode)> = vec![];
        for (player_index, textures) in player_textures.iter_mut().enumerate() {
            texture_refs.push((
                &mut textures.board,
                TextureMode::Board(player_index as u32),
            ));
            texture_refs.push((
                &mut textures.background,
                TextureMode::Background(player_index as u32),
            ));
        }

        let paused_screen =
            PausedScreen::new(&mut self.canvas, &texture_creator, window_size)?;

        let mut frame_rate = FrameRate::new();

        for player in 0..players {
            let cells = fixture.player(player).game().stage_intro_cells();
            themes.animate_next_stage(player, &cells);
        }

        fg_particles.clear();
        bg_particles.clear();
        bg_particles.add_source(self.orbit_particle_source());

        themes.sync_music(fixture.leading_player(), fixture.state(), is_single_player)?;

        loop {
            let delta = frame_rate.update()?;
            fixture.unset_flags();

            let mut to_emit_particles: Vec<PlayerTargetedParticles> = vec![];

            // a stage boundary was crossed this frame: re-evaluate which player the music follows
            let mut stage_changed = false;

            // events tagged with the player that caused them (None for match-wide events) so
            // sound effects are routed through that player's theme
            let mut events: Vec<(Option<u32>, GameEvent)> = vec![];
            for key in inputs.update(delta, self.event_pump.poll_iter()) {
                if let Some(player) = key.player() {
                    if themes.is_pause_required_for_animation(player) {
                        if themes.maybe_dismiss_interstitial(player) {
                            let game = fixture.player_mut(player).game_mut();
                            game.next_stage()?;
                            stage_changed = true;

                            if settings.players[player as usize].theme_mode == ThemeMode::All {
                                // only this player advances to their next theme
                                events.push((Some(player), GameEvent::NextTheme));
                            } else if is_single_player {
                                // single player was playing next-stage music; resume game music.
                                // multiplayer game music never stopped (stage clear is a jingle).
                                themes.music_audio().play_game_music()?;
                            }
                            let cells = game.stage_intro_cells();
                            themes.animate_next_stage(player, &cells);
                        } else {
                            themes.maybe_dismiss_game_over();
                        }
                        // animating, ignore all player game input
                        continue;
                    }
                }

                match key {
                    GameInputKey::MoveLeft { player } => fixture.mut_game(player, |g| g.left()),
                    GameInputKey::MoveRight { player } => fixture.mut_game(player, |g| g.right()),
                    GameInputKey::SoftDrop { player } => {
                        fixture.mut_game(player, |g| g.set_soft_drop(true))
                    }
                    GameInputKey::HardDrop { player } => {
                        fixture.mut_game(player, |g| g.hard_drop())
                    }
                    GameInputKey::RotateClockwise { player } => {
                        fixture.mut_game(player, |g| g.rotate(true))
                    }
                    GameInputKey::RotateAnticlockwise { player } => {
                        fixture.mut_game(player, |g| g.rotate(false))
                    }
                    GameInputKey::Hold { player } => fixture.mut_game(player, |g| g.hold()),
                    GameInputKey::Pause => {
                        if matches!(fixture.state(), MatchState::Normal | MatchState::Paused) {
                            if let Some(paused) = fixture.toggle_paused() {
                                let event = if paused {
                                    GameEvent::Paused
                                } else {
                                    GameEvent::UnPaused
                                };
                                events.push((None, event));
                            }
                        } else {
                            return Ok(PostGameAction::ReturnToMenu);
                        }
                    }
                    GameInputKey::ReturnToMenu => return Ok(PostGameAction::ReturnToMenu),
                    GameInputKey::Quit => return Ok(PostGameAction::Quit),
                    GameInputKey::NextTheme => {
                        if settings.rules.allow_manual_theme_change() {
                            events.push((None, GameEvent::NextTheme))
                        }
                    }
                }
            }

            match fixture.state() {
                MatchState::GameOver {
                    high_score: Some(high_score),
                } if themes.is_all_post_game_animation_complete() => {
                    // start high score entry
                    return Ok(PostGameAction::NewHighScore(high_score));
                }
                MatchState::GameOver { high_score } if themes.is_any_game_over_dismissed() => {
                    return if let Some(high_score) = high_score {
                        // start high score entry
                        Ok(PostGameAction::NewHighScore(high_score))
                    } else {
                        Ok(PostGameAction::ReturnToMenu)
                    };
                }
                MatchState::Normal => {
                    for player in fixture.players.iter_mut() {
                        let player_id = player.player();
                        if themes.is_pause_required_for_animation(player_id)
                            || themes.is_fading(player_id)
                        {
                            continue;
                        }

                        let mut skip_update = false;
                        let game = player.game_mut();
                        let mut player_events = vec![];
                        player_events.extend(game.drain_events());
                        // pre-update actions
                        for event in player_events.iter() {
                            if let GameEvent::HardDrop {
                                cells,
                                dropped_rows,
                            } = event
                            {
                                themes.animate_hard_drop(player_id, cells, *dropped_rows);
                                skip_update = true;
                            }
                        }

                        if !skip_update {
                            game.update(delta);
                            player_events.extend(game.drain_events());
                        }
                        events.extend(player_events.into_iter().map(|e| (Some(player_id), e)));
                    }
                }
                _ => {}
            }

            // update animations
            if !fixture.state().is_paused() {
                let animation_events = themes.update_animations(delta);
                for event in animation_events.into_iter() {
                    match event {
                        AnimationEvent::Finished { player, animation }
                            if animation == AnimationType::Spawn =>
                        {
                            events.push((Some(player), GameEvent::Spawned));
                        }
                        _ => {}
                    }
                }
            }

            // post-update events; a seamless stage change may queue a theme change
            let mut deferred_events: Vec<(Option<u32>, GameEvent)> = vec![];
            for (event_player, event) in events {
                // sound effects play through the theme of the player that caused them;
                // match-wide events (pause etc.) go through the theme whose music is playing
                let (audio, clear_class) = match event_player {
                    Some(player) => (
                        themes.theme(player).audio(),
                        fixture.player(player).game().clear_class(&event),
                    ),
                    None => (themes.music_audio(), 0),
                };
                audio.receive_event(&event, clear_class)?;
                if let Some(player) = event_player {
                    let game = fixture.player(player).game();
                    if let Some(emit) = themes
                        .theme(player)
                        .scene(game.speed_index())
                        .emit_particles(player, &event, &game.spawn_cells())
                    {
                        to_emit_particles.push(emit);
                    }
                }
                match (event_player, event) {
                    (Some(player), GameEvent::StageComplete) => {
                        if fixture.next_stage_ends_match(player) {
                            fixture.set_winner(player);
                        } else {
                            match fixture.player(player).game().stage_transition() {
                                StageTransition::Interstitial => {
                                    if is_single_player {
                                        themes.music_audio().play_next_stage_music()?;
                                    } else {
                                        themes.theme(player).audio().play_stage_complete_jingle()?;
                                    }
                                    themes.animate_interstitial(player);
                                }
                                StageTransition::Seamless => {
                                    fixture.player_mut(player).game_mut().next_stage()?;
                                    stage_changed = true;
                                    if settings.players[player as usize].theme_mode == ThemeMode::All {
                                        deferred_events.push((Some(player), GameEvent::NextTheme));
                                    }
                                }
                            }
                        }
                    }
                    (Some(player), GameEvent::GameOver) => {
                        if is_single_player {
                            // single player is a simple game over
                            themes.animate_game_over(player);
                            fixture.maybe_set_game_over();
                            themes.music_audio().play_game_over_music()?;
                        } else {
                            for maybe_winner in 0..players {
                                if maybe_winner != player {
                                    fixture.set_winner(maybe_winner);
                                }
                            }
                        }
                    }
                    (Some(player), GameEvent::Clear { cells, .. }) => {
                        themes.animate_destroy(player, &cells);
                    }
                    (Some(player), GameEvent::AttackSent(attack)) => {
                        fixture.send_attack(player, attack);
                    }
                    (Some(player), GameEvent::Lock { cells, dropped }) => {
                        if dropped {
                            themes.animate_impact(player);
                        }
                        themes.animate_lock(player, &cells);
                    }
                    (Some(player), GameEvent::Spawn { piece, is_hold, .. }) => {
                        themes.animate_spawn(player, piece, is_hold);
                    }
                    (Some(player), GameEvent::NextTheme) => {
                        themes.fade_into_next_theme(player, &mut self.canvas)?
                    }
                    (None, GameEvent::NextTheme) => {
                        themes.fade_all_into_next_theme(&mut self.canvas)?
                    }
                    _ => {}
                }
            }
            for (player, _) in deferred_events {
                if let Some(player) = player {
                    themes.fade_into_next_theme(player, &mut self.canvas)?;
                }
            }

            // check for a match winner
            if let Some(winner) = fixture.check_for_winning_player() {
                if fixture.maybe_set_game_over() {
                    stage_changed = true;
                    themes.animate_victory(winner);
                    if let Some(emit) = themes
                        .theme(winner)
                        .scene(fixture.player(winner).game().speed_index())
                        .emit_particles(winner, &GameEvent::Victory, &[])
                    {
                        to_emit_particles.push(emit);
                    }
                    for pid in 0..players {
                        if pid != winner {
                            themes.animate_game_over(pid);
                        }
                    }
                    // the winner's theme music is the victory music; sync starts it if the
                    // music is moving to a new theme, otherwise start it explicitly
                    if !themes.sync_music(
                        fixture.leading_player(),
                        fixture.state(),
                        is_single_player,
                    )? {
                        themes.music_audio().play_victory_music()?;
                    }
                }
            }

            // music follows the winning player, re-evaluated between stages
            themes.sync_music(
                if stage_changed {
                    fixture.leading_player()
                } else {
                    None
                },
                fixture.state(),
                is_single_player,
            )?;

            // update particles
            if !fixture.state().is_paused() {
                fg_particles.update(delta);

                if themes.render_scene_particles() {
                    bg_particles.update(delta);
                }
            }
            for emit in to_emit_particles.into_iter() {
                fg_particles.add_source(emit.into_source(&themes, &self.particle_scale));
            }

            // clear
            self.canvas.set_draw_color(Color::BLACK); // TODO
            self.canvas.clear();

            // draw scene
            let games = fixture.players.iter().map(|p| p.game()).collect::<Vec<&G>>();
            themes.draw_scene(&mut self.canvas, &games)?;

            // draw bg particles, clipped to the players on a particle scene
            themes.draw_scene_particles(&mut self.canvas, bg_particles)?;

            // draw the game
            self.canvas
                .with_multiple_texture_canvas(
                    texture_refs.iter(),
                    |texture_canvas, texture_mode| match texture_mode {
                        TextureMode::Background(player_id) => {
                            let player = fixture.player(*player_id);
                            let animations = themes.player_animations(*player_id);
                            themes
                                .theme(*player_id)
                                .draw_background(texture_canvas, player.game(), animations)
                                .unwrap();
                        }
                        TextureMode::Board(player_id) => {
                            let player = fixture.player(*player_id);
                            let animations = themes.player_animations(*player_id);
                            themes
                                .theme(*player_id)
                                .draw_board(texture_canvas, player.game(), animations)
                                .unwrap();
                        }
                    },
                )
                .map_err(|e| e.to_string())?;

            themes.draw_players(&mut self.canvas, &mut texture_refs, delta)?;

            // fg particles
            fg_particles.draw(&mut self.canvas)?;

            if fixture.state().is_paused() {
                paused_screen.draw(&mut self.canvas)?;
            }

            self.canvas.present();
        }
    }
}

