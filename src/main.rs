use std::str::FromStr;
use num_format::Locale::ga;
use sdl2::image::{InitFlag as ImageInitFlag, Sdl2ImageContext};
use sdl2::mixer::{InitFlag as MixerInitFlag, AUDIO_S16LSB, DEFAULT_CHANNELS, AUDIO_S32SYS, DEFAULT_FORMAT, DEFAULT_FREQUENCY};
use sdl2::sys::mixer::MIX_CHANNELS;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::{AudioSubsystem, EventPump, Sdl};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::ttf::Sdl2TtfContext;
use crate::build_info::{APP_NAME, nice_app_name};
use crate::config::{Config, VideoMode};
use crate::frame_rate::FrameRate;
use crate::game::event::GameEvent;
use crate::game::GameSpeed;
use crate::game::rules::{GameConfig, MatchRules, MatchThemes};
use crate::game_input::{GameInputContext, GameInputKey};
use crate::high_score::event::HighScoreEntryEvent;
use crate::high_score::NewHighScore;
use crate::high_score::render::HighScoreRender;
use crate::high_score::table::HighScoreTable;
use crate::icon::app_icon;
use crate::menu::{Menu, MenuAction, MenuItem};
use crate::menu::sound::MenuSound;
use crate::menu_input::{MenuInputContext, MenuInputKey};
use crate::particles::Particles;
use crate::particles::prescribed::{PlayerTargetedParticles, prescribed_fireworks, prescribed_orbit, prescribed_vitamin_race};
use crate::particles::render::ParticleRender;
use crate::particles::source::ParticleSource;
use crate::player::{Match, MatchState};
use crate::theme::all::AllThemes;
use crate::theme::pause::PausedScreen;
use crate::themes::{PlayerTextures, TextureMode, ThemeContext};

mod game;
mod theme;
mod config;
mod build_info;
mod game_input;
mod menu_input;
mod high_score;
mod player;
mod themes;
mod scale;
mod frame_rate;
mod animate;
mod font;
mod menu;
mod icon;
mod particles;

const MAX_PLAYERS: u32 = 2;
const MAX_PARTICLES_PER_PLAYER: usize = 100000;
const MAX_BACKGROUND_PARTICLES: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MainMenuAction {
    Start,
    ViewHighScores,
    Back,
    Quit
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PostGameAction {
    NewHighScore(NewHighScore),
    ReturnToMenu,
    Quit
}

struct DrRustario {
    config: Config,
    _sdl: Sdl,
    ttf: Sdl2TtfContext,
    _image: Sdl2ImageContext,
    canvas: WindowCanvas,
    event_pump: EventPump,
    _audio: AudioSubsystem,
    menu_sound: MenuSound,
    game_config: GameConfig,
    particle_scale: particles::scale::Scale,
}

impl DrRustario {
    pub fn new() -> Result<Self, String> {
        let config = Config::load()?;
        let sdl = sdl2::init()?;
        let image = sdl2::image::init(ImageInitFlag::PNG)?;
        let video = sdl.video()?;
        let ttf = sdl2::ttf::init().map_err(|e| e.to_string())?;

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

        let mut window_builder = video.window(APP_NAME, width, height);
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
            .allow_highdpi()
            .build()
            .map_err(|e| e.to_string())?;

        window.set_icon(app_icon()?);

        let canvas_builder = window
            .into_canvas()
            .target_texture()
            .accelerated();

        let canvas = if config.video.vsync {
            canvas_builder.present_vsync()
        } else {
            canvas_builder
        }
            .build()
            .map_err(|e| e.to_string())?;

        let event_pump = sdl.event_pump()?;

        let audio = sdl.audio()?;
        sdl2::mixer::open_audio(44_100, DEFAULT_FORMAT, DEFAULT_CHANNELS, 512)?;
        let _mixer_context = sdl2::mixer::init(MixerInitFlag::OGG)?;
        sdl2::mixer::allocate_channels((MAX_PLAYERS * MIX_CHANNELS) as i32);
        sdl2::mixer::Music::set_volume(config.audio.music_volume());
        let menu_sound = MenuSound::new(config.audio)?;

        Ok(Self {
            config,
            _sdl: sdl,
            ttf,
            _image: image,
            canvas,
            event_pump,
            _audio: audio,
            menu_sound,
            game_config: Default::default(),
            particle_scale: particles::scale::Scale::new((width, height)),
        })
    }

    fn vitamin_race_particle_source(&self) -> Box<dyn ParticleSource> {
        let (window_width, window_height) = self.canvas.window().size();
        prescribed_vitamin_race(
            Rect::new(0, 0, window_width, window_height),
            &self.particle_scale,
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

    pub fn title_menu(&mut self, particles: &mut ParticleRender) -> Result<MainMenuAction, String> {
        const PLAYERS: &str = "players";
        const HIGH_SCORES: &str = "high scores";
        const START: &str = "start";
        const QUIT: &str = "quit";

        let texture_creator = self.canvas.texture_creator();
        let inputs = MenuInputContext::new(self.config.input);
        let mut menu = Menu::new(
            vec![
                MenuItem::select_list(
                    PLAYERS,
                    vec!["1".to_string(), "2".to_string()],
                    self.game_config.players() as usize - 1,
                ),
                MenuItem::select(HIGH_SCORES),
                MenuItem::select(START),
                MenuItem::select(QUIT),
            ],
            &mut self.canvas,
            &self.ttf,
            &texture_creator,
            nice_app_name(),
            None
        )?;

        particles.clear();
        particles.add_source(self.vitamin_race_particle_source());

        let mut frame_rate = FrameRate::new();
        self.menu_sound.play_title_music()?;
        loop {
            let delta = frame_rate.update()?;
            for key in inputs.parse(self.event_pump.poll_iter()).into_iter() {
                if key == MenuInputKey::Quit {
                    return Ok(MainMenuAction::Quit);
                }
                self.menu_sound.play_chime()?;
                match menu.read_key(key) {
                    None => match key {
                        MenuInputKey::Start => return Ok(MainMenuAction::Start),
                        MenuInputKey::Back => return Ok(MainMenuAction::Back),
                        _ => {}
                    },
                    Some((name, action)) => match name {
                        PLAYERS => {
                            let players = action.parse::<u32>().unwrap();
                            self.game_config.set_players(players);
                            self.game_config.set_rules(MatchRules::default_by_players(players));
                        },
                        HIGH_SCORES => return Ok(MainMenuAction::ViewHighScores),
                        START => return Ok(MainMenuAction::Start),
                        QUIT => return Ok(MainMenuAction::Back),
                        _ => {}
                    },
                }
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

    pub fn main_menu(&mut self, particles: &mut ParticleRender) -> Result<MainMenuAction, String> {
        const THEMES: &str = "themes";
        const MODE: &str = "mode";
        const LEVEL: &str = "level";
        const SPEED: &str = "speed";
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
                MatchThemes::names().into_iter().map(|s| s.to_string()).collect(),
                self.game_config.themes() as usize,
            ),
            MenuItem::select_list(
                MODE,
                modes.iter().map(|m| m.name()).collect(),
                modes.iter().position(|&m| m == self.game_config.rules()).unwrap()
            ),
            MenuItem::select_list(
                LEVEL,
                (0..=30).map(|i| i.to_string()).collect(),
                self.game_config.virus_level() as usize,
            ),
            MenuItem::select_list(
                SPEED,
                GameSpeed::names().into_iter().map(|s| s.to_string()).collect(),
                self.game_config.speed() as usize,
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
            &self.ttf,
            &texture_creator,
            nice_app_name(),
            Some(subtitle)
        )?;

        particles.clear();
        particles.add_source(self.vitamin_race_particle_source());

        let mut frame_rate = FrameRate::new();
        self.menu_sound.play_menu_music()?;
        loop {
            let delta = frame_rate.update()?;

            for key in inputs.parse(self.event_pump.poll_iter()).into_iter() {
                if key == MenuInputKey::Quit {
                    return Ok(MainMenuAction::Quit);
                }
                self.menu_sound.play_chime()?;
                match menu.read_key(key) {
                    None => match key {
                        MenuInputKey::Start => return Ok(MainMenuAction::Start),
                        MenuInputKey::Back => return Ok(MainMenuAction::Back),
                        _ => {}
                    },
                    Some((name, action)) => match name {
                        THEMES => self.game_config.set_themes(MatchThemes::from_str(action).unwrap()),
                        MODE => {
                            let mode_index = modes.iter().position(|&m| m.name() == action).unwrap();
                            self.game_config.set_rules(modes[mode_index])
                        }
                        LEVEL => self.game_config.set_virus_level(action.parse::<u32>().unwrap()),
                        SPEED => self.game_config.set_speed(GameSpeed::from_str(action).unwrap()),
                        START => return Ok(MainMenuAction::Start),
                        BACK => return Ok(MainMenuAction::Back),
                        _ => {}
                    },
                }
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
            &self.ttf,
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

    pub fn new_high_score(&mut self, new_high_score: NewHighScore, particles: &mut ParticleRender) -> Result<(), String> {
        let texture_creator = self.canvas.texture_creator();
        let inputs = MenuInputContext::new(self.config.input);
        let high_scores = HighScoreTable::load()?;
        if high_scores.entries().is_empty() {
            return Ok(());
        }

        let mut table = HighScoreRender::new(
            high_scores,
            &self.ttf,
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
        all_themes: &AllThemes,
        fg_particles: &mut ParticleRender,
        bg_particles: &mut ParticleRender
    ) -> Result<PostGameAction, String> {
        let texture_creator = self.canvas.texture_creator();
        let mut inputs = GameInputContext::new(self.config.input);
        let mut fixture = Match::new(self.game_config);
        let window_size = self.canvas.window().size();
        let mut themes = ThemeContext::new(all_themes, &texture_creator, self.game_config, window_size)?;
        let mut player_textures = (0..self.game_config.players())
            .map(|_| {
                PlayerTextures::new(
                    &texture_creator,
                    themes.max_background_size(),
                    themes.max_bottle_size(),
                ).unwrap()
            })
            .collect::<Vec<PlayerTextures>>();

        // push mut refs of all textures and their render modes into a single vector so we can render to texture in one loop
        let mut texture_refs: Vec<(&mut Texture, TextureMode)> = vec![];
        for (player_index, textures) in player_textures.iter_mut().enumerate() {
            texture_refs.push((&mut textures.bottle, TextureMode::Bottle(player_index as u32)));
            texture_refs.push((
                &mut textures.background,
                TextureMode::Background(player_index as u32),
            ));
        }

        let paused_screen =
            PausedScreen::new(&mut self.canvas, &self.ttf, &texture_creator, window_size)?;

        let mut frame_rate = FrameRate::new();

        for player in 0..self.game_config.players() {
            let viruses = fixture.player(player).game().viruses();
            themes.animate_next_level(player, viruses.as_slice());
        }

        fg_particles.clear();
        bg_particles.clear();
        bg_particles.add_source(self.orbit_particle_source());

        themes.theme().audio().play_game_music()?;

        let mut max_virus_level = self.game_config.virus_level();

        loop {
            let delta = frame_rate.update()?;
            fixture.unset_flags();

            let mut to_emit_particles: Vec<PlayerTargetedParticles> = vec![];

            let mut events = vec![];
            for key in inputs.update(delta, self.event_pump.poll_iter()) {
                if let Some(player) = key.player() {
                    if themes.current().is_pause_required_for_animation(player) {
                        if themes.maybe_dismiss_next_level_interstitial(player) {
                            let game = fixture.player_mut(player).game_mut();
                            game.next_level()?;

                            let next_level = game.virus_level();
                            let is_first_to_next_level = next_level > max_virus_level;
                            max_virus_level = next_level;

                            if self.game_config.themes() == MatchThemes::All {
                                if is_first_to_next_level {
                                    events.push(GameEvent::NextTheme);
                                }
                            } else if self.game_config.is_single_player() {
                                // only start game music here if on single player and not switching themes
                                themes.theme().audio().play_game_music()?;
                            }
                            themes.animate_next_level(player, game.viruses().as_slice());
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
                    GameInputKey::SoftDrop { player } => fixture.mut_game(player, |g| g.set_soft_drop(true)),
                    GameInputKey::HardDrop { player } => fixture.mut_game(player, |g| g.hard_drop()),
                    GameInputKey::RotateClockwise { player } => fixture.mut_game(player, |g| g.rotate(true)),
                    GameInputKey::RotateAnticlockwise { player } => fixture.mut_game(player, |g| g.rotate(false)),
                    GameInputKey::Hold { player } => fixture.mut_game(player, |g| g.hold()),
                    GameInputKey::Pause => {
                        if matches!(fixture.state(), MatchState::Normal | MatchState::Paused) {
                            fixture.toggle_paused().map(|e| events.push(e));
                        } else {
                            return Ok(PostGameAction::ReturnToMenu);
                        }
                    },
                    GameInputKey::ReturnToMenu => return Ok(PostGameAction::ReturnToMenu),
                    GameInputKey::Quit => return Ok(PostGameAction::Quit),
                    GameInputKey::NextTheme => {
                        if self.game_config.rules().allow_manual_theme_change() {
                            events.push(GameEvent::NextTheme)
                        }
                    },
                }
            }

            match fixture.state() {
                MatchState::GameOver { high_score: Some(high_score) } if themes.is_all_post_game_animation_complete() => {
                    // start high score entry
                    return Ok(PostGameAction::NewHighScore(high_score));
                }
                MatchState::GameOver { high_score } if themes.is_any_game_over_dismissed() => {
                    return if let Some(high_score) = high_score {
                        // start high score entry
                        Ok(PostGameAction::NewHighScore(high_score))
                    } else {
                        Ok(PostGameAction::ReturnToMenu)
                    }
                }
                MatchState::Normal if !themes.is_fading() => {
                    for player in fixture.players.iter_mut() {
                        if themes.current().is_pause_required_for_animation(player.player()) {
                            continue;
                        }

                        let mut skip_update = false;
                        let game = player.game_mut();
                        game.consume_events(&mut events);
                        // pre-update actions
                        for event in events.iter() {
                            match event {
                                GameEvent::HardDrop { player, vitamins, dropped_rows } => {
                                    themes.animate_hard_drop(*player, *vitamins, *dropped_rows);
                                    skip_update = true;
                                }
                                _ => {}
                            }
                        }

                        if !skip_update {
                            game.update(delta);
                            game.consume_events(&mut events);
                        }
                    }
                }
                _ => {}
            }

            // post-update events
            for event in events {
                themes.theme().audio().receive_event(event.clone())?;
                if let Some(emit) = themes.theme().scene(self.game_config.speed()).emit_particles(event.clone()) {
                    to_emit_particles.push(emit);
                }
                match event {
                    GameEvent::LevelComplete { player } => {
                        if fixture.next_level_ends_match(player) {
                            fixture.set_winner(player);
                        } else {
                            if self.game_config.is_single_player() {
                                themes.theme().audio().play_next_level_music()?;
                            } else {
                                themes.theme().audio().play_next_level_jingle()?;
                            }
                            themes.animate_next_level_interstitial(player);
                        }
                    },
                    GameEvent::GameOver { player } => {
                        if self.game_config.is_single_player() {
                            // single player is a simple game over
                            themes.animate_game_over(player);
                            fixture.maybe_set_game_over();
                            themes.theme().audio().play_game_over_music()?;
                        } else {
                            for maybe_winner in 0..self.game_config.players() {
                                if maybe_winner != player {
                                    fixture.set_winner(maybe_winner);
                                }
                            }
                        }
                    },
                    GameEvent::Destroy { player, blocks, .. } => {
                        themes.animate_destroy(player, blocks);
                    }
                    GameEvent::SendGarbage { player, garbage } => {
                        fixture.send_garbage(player, garbage);
                    },
                    GameEvent::Lock { player, vitamins, hard_or_soft_dropped } => {
                        if hard_or_soft_dropped {
                            themes.animate_impact(player);
                        }
                        themes.animate_lock(player, vitamins);
                    }
                    GameEvent::Spawn { player, shape, is_hold, .. } => {
                        themes.animate_spawn(player, shape, is_hold);
                    },
                    GameEvent::NextTheme => {
                        themes.fade_into_next_theme(
                            &mut self.canvas,
                            fixture.state(),
                            self.game_config.is_single_player()
                        )?;
                    }
                    _ => {}
                }
            }

            // check for a match winner
            if let Some(winner) = fixture.check_for_winning_player() {
                if fixture.maybe_set_game_over() {
                    themes.animate_victory(winner);
                    for pid in 0..self.game_config.players() {
                        if pid != winner {
                            themes.animate_game_over(pid);
                        }
                    }
                    themes.theme().audio().play_victory_music()?;
                }
            }

            // update themes
            if !fixture.state().is_paused() {
                themes.update(delta);
            }

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
            themes.draw_scene(&mut self.canvas, self.game_config.speed())?;

            // draw bg particles
            if themes.render_scene_particles() {
                bg_particles.draw(&mut self.canvas)?;
            }

            // draw the game
            self.canvas
                .with_multiple_texture_canvas(
                    texture_refs.iter(),
                    |texture_canvas, texture_mode| match texture_mode {
                        TextureMode::Background(player_id) => {
                            let player = fixture.player(*player_id);
                            let animations = themes.player_animations(*player_id);
                            themes
                                .theme()
                                .draw_background(texture_canvas, &player.game(), animations)
                                .unwrap();
                        }
                        TextureMode::Bottle(player_id) => {
                            let player = fixture.player(*player_id);
                            let animations = themes.player_animations(*player_id);
                            themes
                                .theme()
                                .draw_bottle(texture_canvas, &player.game(), animations)
                                .unwrap();
                        }
                        _ => {}
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
    let mut dr_rustario = DrRustario::new()?;;
    let texture_creator = dr_rustario.canvas.texture_creator();
    let all_themes = AllThemes::new(
        &mut dr_rustario.canvas,
        &texture_creator,
        &dr_rustario.ttf,
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
        all_themes.all(),
    )?;

    'title: loop {
        match dr_rustario.title_menu(&mut bg_particles)? {
            MainMenuAction::Start => {
                'select: loop {
                    match dr_rustario.main_menu(&mut bg_particles)? {
                        MainMenuAction::Start => {
                            match dr_rustario.game(&all_themes, &mut fg_particles, &mut bg_particles)? {
                                PostGameAction::NewHighScore(high_score) => dr_rustario.new_high_score(high_score, &mut bg_particles)?,
                                PostGameAction::ReturnToMenu => (),
                                PostGameAction::Quit => return Ok(()),
                            }
                        }
                        MainMenuAction::Back => break 'select,
                        MainMenuAction::Quit => return Ok(()),
                        _ => unreachable!()
                    }
                }
            }
            MainMenuAction::ViewHighScores => dr_rustario.view_high_score(&mut bg_particles)?,
            MainMenuAction::Back => break 'title,
            MainMenuAction::Quit => return Ok(())
        }
    }
    Ok(())
}
