#![windows_subsystem = "windows"]

use engine::animate::event::{AnimationEvent, AnimationType};
use crate::config::{Config, VideoMode};
use crate::frame_rate::FrameRate;
use crate::game::event::GameEvent;
use crate::game::random::{random, RandomMode};
use crate::game::rules::{GameConfig, MatchRules, MatchThemes};
use crate::game::{Game, GameSpeed};
use crate::theme::{all_themes, race_themes};
use engine::game::{Game as _, PlacedCell};
use engine::particles::prescribed::{
    prescribed_fireworks, prescribed_orbit, prescribed_piece_race, PlayerTargetedParticles,
};
use engine::particles::render::ParticleRender;
use engine::particles::source::ParticleSource;
use engine::particles::Particles;
use engine::render::context::{PlayerTextures, TextureMode, ThemeContext};
use engine::render::pause::PausedScreen;
use engine::render::{GameRender, Theme};
use engine::session::{Match, MatchState};
use crate::game_input::{GameInputContext, GameInputKey};
use crate::high_score::event::HighScoreEntryEvent;
use crate::high_score::render::HighScoreRender;
use crate::high_score::table::HighScoreTable;
use crate::high_score::NewHighScore;
use crate::icon::app_icon;
use crate::menu::sound::MenuSound;
use crate::menu::{Menu, MenuItem};
use crate::menu_input::{MenuInputContext, MenuInputKey};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::{EventPump, Sdl};
use std::str::FromStr;

mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));

    pub fn nice_app_name() -> String {
        titlecase::titlecase(&PKG_NAME.replace("-", ". "))
    }
}
mod game;
mod render;
mod theme;

use engine::{audio, config, frame_rate, game_input, high_score, icon, menu, menu_input};

#[cfg(not(feature = "retro_handheld"))]
const MAX_PLAYERS: u32 = 2;

#[cfg(feature = "retro_handheld")]
const MAX_PLAYERS: u32 = 1;

/// Simultaneous sound effects per player (SDL_mixer's default channel count).
const EFFECT_CHANNELS_PER_PLAYER: u32 = 8;

const MAX_PARTICLES_PER_PLAYER: usize = 100000;
const MAX_BACKGROUND_PARTICLES: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MainMenuAction {
    Start,
    ViewHighScores,
    Back,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PostGameAction {
    NewHighScore(NewHighScore),
    ReturnToMenu,
    Quit,
}

struct DrRustario {
    config: Config,
    _sdl: Sdl,
    canvas: WindowCanvas,
    event_pump: EventPump,
    _audio: audio::Audio,
    menu_sound: MenuSound,
    game_config: GameConfig,
    particle_scale: engine::particles::scale::Scale,
}

impl DrRustario {
    pub fn new() -> Result<Self, String> {
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

        let mut window_builder = video.window(build_info::PKG_NAME, width, height);
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

        window.set_icon(app_icon(include_bytes!("../icon.png"))?);

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
            MAX_PLAYERS * EFFECT_CHANNELS_PER_PLAYER,
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
            game_config: Default::default(),
            particle_scale: engine::particles::scale::Scale::new((width, height)),
        })
    }

    fn vitamin_race_particle_source(&self, themes: &[Theme]) -> Box<dyn ParticleSource> {
        let (window_width, window_height) = self.canvas.window().size();
        prescribed_piece_race(
            Rect::new(0, 0, window_width, window_height),
            &self.particle_scale,
            &race_themes(themes),
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

    pub fn title_menu(
        &mut self,
        all_themes: &[Theme],
        particles: &mut ParticleRender,
    ) -> Result<MainMenuAction, String> {
        const PLAYERS: &str = "players";
        const HIGH_SCORES: &str = "high scores";
        const START: &str = "start";
        const QUIT: &str = "quit";

        let texture_creator = self.canvas.texture_creator();
        let inputs = MenuInputContext::new(self.config.input);

        let mut menu_items = vec![
            MenuItem::select(HIGH_SCORES),
            MenuItem::select(START),
            MenuItem::select(QUIT),
        ];

        if MAX_PLAYERS > 1 {
            menu_items.insert(
                0,
                MenuItem::select_list(
                    PLAYERS,
                    (1..=MAX_PLAYERS)
                        .into_iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<String>>(),
                    self.game_config.players() as usize - 1,
                )
            )
        }

        let mut menu = Menu::new(
            menu_items,
            &mut self.canvas,
            &texture_creator,
            build_info::nice_app_name(),
            None,
        )?;

        particles.clear();
        particles.add_source(self.vitamin_race_particle_source(all_themes));

        let mut frame_rate = FrameRate::new();
        self.menu_sound.play_title_music()?;
        loop {
            let delta = frame_rate.update()?;
            for key in inputs.parse(self.event_pump.poll_iter()).into_iter() {
                if key == MenuInputKey::Quit {
                    return Ok(MainMenuAction::Quit);
                }

                match menu.read_key(key) {
                    None => match key {
                        MenuInputKey::Start => {
                            self.menu_sound.play_select()?;
                            return Ok(MainMenuAction::Start);
                        }
                        MenuInputKey::Back => return Ok(MainMenuAction::Back),
                        _ => {}
                    },
                    Some((name, action)) => match name {
                        PLAYERS => {
                            let players = action.parse::<u32>().unwrap();
                            self.game_config.set_players(players);
                            self.game_config
                                .set_rules(MatchRules::default_by_players(players));
                        }
                        HIGH_SCORES => return Ok(MainMenuAction::ViewHighScores),
                        START => {
                            self.menu_sound.play_select()?;
                            return Ok(MainMenuAction::Start);
                        }
                        QUIT => return Ok(MainMenuAction::Back),
                        _ => {}
                    },
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

    pub fn main_menu(
        &mut self,
        all_themes: &[Theme],
        particles: &mut ParticleRender,
    ) -> Result<MainMenuAction, String> {
        const THEMES: &str = "themes";
        const MODE: &str = "mode";
        const LEVEL: &str = "level";
        const SPEED: &str = "speed";
        const RANDOM: &str = "random";
        const START: &str = "start";
        const BACK: &str = "back";

        let modes = if self.game_config.is_single_player() {
            MatchRules::SINGLE_PLAYER_MODES.to_vec()
        } else {
            MatchRules::VS_MODES.to_vec()
        };

        let texture_creator = self.canvas.texture_creator();
        let inputs = MenuInputContext::new(self.config.input);

        let menu_items = vec![
            MenuItem::select_list(
                THEMES,
                MatchThemes::names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                self.game_config.themes() as usize,
            ),
            MenuItem::select_list(
                MODE,
                modes.iter().map(|m| m.name("level")).collect(),
                modes
                    .iter()
                    .position(|&m| m == self.game_config.rules())
                    .unwrap(),
            ),
            MenuItem::select_list(
                LEVEL,
                (0..=30).map(|i| i.to_string()).collect(),
                self.game_config.virus_level() as usize,
            ),
            MenuItem::select_list(
                SPEED,
                GameSpeed::names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                self.game_config.speed() as usize,
            ),
            MenuItem::select_list(
                RANDOM,
                RandomMode::names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                self.game_config.random() as usize,
            ),
            MenuItem::select(START),
            MenuItem::select(BACK),
        ];
        let subtitle = if self.game_config.is_single_player() {
            "single player".to_string()
        } else {
            format!("{}-player vs.", self.game_config.players())
        };
        let mut menu = Menu::new(
            menu_items,
            &mut self.canvas,
            &texture_creator,
            build_info::nice_app_name(),
            Some(subtitle),
        )?;

        particles.clear();
        particles.add_source(self.vitamin_race_particle_source(all_themes));

        let mut frame_rate = FrameRate::new();
        self.menu_sound.play_menu_music()?;
        loop {
            let delta = frame_rate.update()?;

            for key in inputs.parse(self.event_pump.poll_iter()).into_iter() {
                if key == MenuInputKey::Quit {
                    return Ok(MainMenuAction::Quit);
                }
                match menu.read_key(key) {
                    None => match key {
                        MenuInputKey::Start => {
                            self.menu_sound.play_select()?;
                            return Ok(MainMenuAction::Start);
                        }
                        MenuInputKey::Back => return Ok(MainMenuAction::Back),
                        _ => {}
                    },
                    Some((name, action)) => match name {
                        THEMES => self
                            .game_config
                            .set_themes(MatchThemes::from_str(action).unwrap()),
                        MODE => {
                            let mode_index =
                                modes.iter().position(|&m| m.name("level") == action).unwrap();
                            self.game_config.set_rules(modes[mode_index])
                        }
                        LEVEL => self
                            .game_config
                            .set_virus_level(action.parse::<u32>().unwrap()),
                        SPEED => self
                            .game_config
                            .set_speed(GameSpeed::from_str(action).unwrap()),
                        RANDOM => self
                            .game_config
                            .set_random(RandomMode::from_str(action).unwrap()),
                        START => return Ok(MainMenuAction::Start),
                        BACK => {
                            self.menu_sound.play_select()?;
                            return Ok(MainMenuAction::Back);
                        }
                        _ => {}
                    },
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

    pub fn view_high_score(&mut self, particles: &mut ParticleRender) -> Result<(), String> {
        let texture_creator = self.canvas.texture_creator();
        let inputs = MenuInputContext::new(self.config.input);
        let high_scores = HighScoreTable::load()?;
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
        new_high_score: NewHighScore,
        particles: &mut ParticleRender,
    ) -> Result<(), String> {
        let texture_creator = self.canvas.texture_creator();
        let inputs = MenuInputContext::new(self.config.input);
        let high_scores = HighScoreTable::load()?;
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
            let mut high_scores = HighScoreTable::load().unwrap();
            high_scores.add_high_score(new_entry);
            high_scores.save()
        } else {
            Ok(())
        }
    }

    pub fn game(
        &mut self,
        all_themes: &[Theme],
        fg_particles: &mut ParticleRender,
        bg_particles: &mut ParticleRender,
    ) -> Result<PostGameAction, String> {
        let texture_creator = self.canvas.texture_creator();
        let mut inputs = GameInputContext::new(self.config.input);
        let game_config = self.game_config;
        let games = random(game_config.players() as usize, game_config.random())
            .into_iter()
            .map(|rand| Game::new(game_config.virus_level(), game_config.speed(), rand))
            .collect::<Result<Vec<Game>, String>>()?;
        let mut fixture = Match::new(games, game_config.rules(), all_themes.len() as u32);
        let window_size = self.canvas.window().size();
        let initial_theme = match game_config.themes() {
            MatchThemes::All | MatchThemes::Nes => 0,
            MatchThemes::Snes => 1,
            MatchThemes::N64 => 2,
            MatchThemes::Particle => 3,
        };
        let mut themes = ThemeContext::new(
            all_themes,
            &texture_creator,
            game_config.players(),
            initial_theme,
            window_size,
            self.config.video,
        )?;
        let mut player_textures = (0..self.game_config.players())
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

        for player in 0..self.game_config.players() {
            themes.animate_next_stage(player, &virus_cells(fixture.player(player).game()));
        }

        fg_particles.clear();
        bg_particles.clear();
        bg_particles.add_source(self.orbit_particle_source());

        themes.sync_music(fixture.leading_player(), fixture.state(), self.game_config.is_single_player())?;

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
                            game.next_level()?;
                            stage_changed = true;

                            if self.game_config.themes() == MatchThemes::All {
                                // only this player advances to their next theme
                                events.push((Some(player), GameEvent::NextTheme));
                            } else if self.game_config.is_single_player() {
                                // single player was playing next-level music; resume game music.
                                // multiplayer game music never stopped (stage clear is a jingle).
                                themes.music_audio().play_game_music()?;
                            }
                            themes.animate_next_stage(player, &virus_cells(game));
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
                        if self.game_config.rules().allow_manual_theme_change() {
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
                        game.consume_events(&mut player_events);
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
                            game.consume_events(&mut player_events);
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

            // post-update events
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
                            if self.game_config.is_single_player() {
                                themes.music_audio().play_next_stage_music()?;
                            } else {
                                themes.theme(player).audio().play_stage_complete_jingle()?;
                            }
                            themes.animate_interstitial(player);
                        }
                    }
                    (Some(player), GameEvent::GameOver) => {
                        if self.game_config.is_single_player() {
                            // single player is a simple game over
                            themes.animate_game_over(player);
                            fixture.maybe_set_game_over();
                            themes.music_audio().play_game_over_music()?;
                        } else {
                            for maybe_winner in 0..self.game_config.players() {
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
                    for pid in 0..self.game_config.players() {
                        if pid != winner {
                            themes.animate_game_over(pid);
                        }
                    }
                    // the winner's theme music is the victory music; sync starts it if the
                    // music is moving to a new theme, otherwise start it explicitly
                    if !themes.sync_music(
                        fixture.leading_player(),
                        fixture.state(),
                        self.game_config.is_single_player(),
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
                self.game_config.is_single_player(),
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
            let games = fixture.players.iter().map(|p| p.game()).collect::<Vec<&Game>>();
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

fn main() -> Result<(), String> {
    engine::app_info::init(engine::app_info::AppInfo {
        name: build_info::PKG_NAME,
        version: build_info::PKG_VERSION,
        authors: build_info::PKG_AUTHORS,
    });

    let mut dr_rustario = DrRustario::new()?;
    let texture_creator = dr_rustario.canvas.texture_creator();
    let all_themes = all_themes(
        &mut dr_rustario.canvas,
        &texture_creator,
        dr_rustario.config,
    )?;

    let mut fg_particles = ParticleRender::new(
        &mut dr_rustario.canvas,
        Particles::new(MAX_PARTICLES_PER_PLAYER * MAX_PLAYERS as usize),
        &texture_creator,
        dr_rustario.particle_scale,
        vec![],
    )?;

    let mut bg_particles = ParticleRender::new(
        &mut dr_rustario.canvas,
        Particles::new(MAX_BACKGROUND_PARTICLES),
        &texture_creator,
        dr_rustario.particle_scale,
        all_themes.iter().collect(),
    )?;

    'title: loop {
        match dr_rustario.title_menu(&all_themes, &mut bg_particles)? {
            MainMenuAction::Start => 'select: loop {
                match dr_rustario.main_menu(&all_themes, &mut bg_particles)? {
                    MainMenuAction::Start => {
                        match dr_rustario.game(&all_themes, &mut fg_particles, &mut bg_particles)? {
                            PostGameAction::NewHighScore(high_score) => {
                                dr_rustario.new_high_score(high_score, &mut bg_particles)?
                            }
                            PostGameAction::ReturnToMenu => (),
                            PostGameAction::Quit => return Ok(()),
                        }
                    }
                    MainMenuAction::Back => break 'select,
                    MainMenuAction::Quit => return Ok(()),
                    _ => unreachable!(),
                }
            },
            MainMenuAction::ViewHighScores => dr_rustario.view_high_score(&mut bg_particles)?,
            MainMenuAction::Back => break 'title,
            MainMenuAction::Quit => return Ok(()),
        }
    }
    Ok(())
}

/// the viruses of a fresh bottle, for the pop-in animation
fn virus_cells(game: &Game) -> Vec<PlacedCell> {
    game.viruses().into_iter().map(PlacedCell::from).collect()
}
