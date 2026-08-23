#![windows_subsystem = "windows"]

mod games;

use crate::games::{AnyGame, Choice, GameKind};
use engine::app::{
    App, MatchSettings, MenuExit, MenuMusic, PlayerSettings, PostGameAction,
    MAX_BACKGROUND_PARTICLES, MAX_PARTICLES_PER_PLAYER,
};
use engine::menu::MenuItem;
use engine::particles::prescribed::RaceTheme;
use engine::render::Theme;
use std::ops::Range;

mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));

    pub fn nice_app_name() -> String {
        titlecase::titlecase(&PKG_NAME.replace("-", " "))
    }
}

#[cfg(not(feature = "retro_handheld"))]
const MAX_PLAYERS: u32 = 2;

#[cfg(feature = "retro_handheld")]
const MAX_PLAYERS: u32 = 1;

const PLAYERS: &str = "players";
const HIGH_SCORES: &str = "high scores";
const START: &str = "start";
const BACK: &str = "back";
const QUIT: &str = "quit";

fn game_item(player: u32) -> String {
    format!("p{} game", player + 1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TitleAction {
    ViewHighScores,
}

/// What the player has picked so far.
struct Selection {
    players: u32,
    games: Vec<Choice>,
    dr_rustario: dr_rustario::options::Options,
    rustris: rustris::options::Options,
}

impl Selection {
    fn set_players(&mut self, players: u32) {
        self.players = players;
        self.dr_rustario.set_players(players);
        self.rustris.set_players(players);
    }

    fn choice(&self, player: u32) -> Choice {
        self.games[player as usize]
    }

    /// the game a player starts on
    fn game(&self, player: u32) -> GameKind {
        self.choice(player).stage(0)
    }

    /// the games in play, first player's first
    fn kinds(&self) -> Vec<GameKind> {
        let mut kinds = vec![];
        for player in 0..self.players {
            for kind in self.choice(player).kinds() {
                if !kinds.contains(&kind) {
                    kinds.push(kind);
                }
            }
        }
        kinds
    }

    fn is_mixed(&self) -> bool {
        self.kinds().len() > 1
    }

    fn is_playlist(&self) -> bool {
        (0..self.players).any(|p| self.choice(p).is_playlist())
    }

    fn theme_mode(&self, kind: GameKind) -> engine::app::ThemeMode {
        match kind {
            GameKind::DrRustario => self.dr_rustario.theme_mode(),
            GameKind::Rustris => self.rustris.theme_mode(),
        }
    }

    fn player_settings(&self, themes: &Themes, kind: GameKind) -> PlayerSettings {
        PlayerSettings {
            themes: themes.range(kind),
            theme_mode: self.theme_mode(kind),
        }
    }

    /// a fresh game of `kind` for one player of a playlist
    fn stage_game(&self, kind: GameKind) -> Result<AnyGame, String> {
        Ok(match kind {
            GameKind::DrRustario => AnyGame::DrRustario(self.dr_rustario.games(1)?.remove(0)),
            GameKind::Rustris => AnyGame::Rustris(self.rustris.games(1).remove(0)),
        })
    }

    /// option names are prefixed with the game in a mixed match
    fn prefix(&self, kind: GameKind) -> String {
        if self.is_mixed() {
            format!("{} ", kind.name())
        } else {
            String::new()
        }
    }

    fn menu_items(&self) -> Vec<MenuItem> {
        let mut items = vec![];
        for (i, kind) in self.kinds().into_iter().enumerate() {
            let prefix = self.prefix(kind);
            // the first game's mode is the match's; the second game keeps its menu short
            let compact = i > 0;
            let game_items = match kind {
                GameKind::DrRustario => self.dr_rustario.menu_items(compact),
                GameKind::Rustris => self.rustris.menu_items(compact),
            };
            items.extend(game_items.into_iter().map(|item| item.with_prefix(&prefix)));
        }
        items
    }

    fn select(&mut self, name: &str, value: &str) {
        for kind in self.kinds() {
            let prefix = self.prefix(kind);
            let Some(name) = name.strip_prefix(&prefix) else {
                continue;
            };
            let handled = match kind {
                GameKind::DrRustario => self.dr_rustario.select(name, value),
                GameKind::Rustris => self.rustris.select(name, value),
            };
            if handled {
                return;
            }
        }
    }

    fn settings(&self, themes: &Themes) -> MatchSettings {
        let rules = match self.game(0) {
            GameKind::DrRustario => self.dr_rustario.rules(),
            GameKind::Rustris => self.rustris.rules(),
        };
        let players = (0..self.players)
            .map(|player| self.player_settings(themes, self.game(player)))
            .collect();
        MatchSettings {
            rules,
            players,
            high_score_key: self.high_score_key(),
            playlist: self.is_playlist(),
        }
    }

    fn high_score_key(&self) -> String {
        if self.is_playlist() {
            "playlist".to_string()
        } else if self.is_mixed() {
            "mixed".to_string()
        } else {
            self.game(0).key().to_string()
        }
    }

    /// one game per player; players on the same game share a seed
    fn games(&self) -> Result<Vec<AnyGame>, String> {
        let mut games: Vec<Option<AnyGame>> = (0..self.players).map(|_| None).collect();
        for kind in self.kinds() {
            let players = (0..self.players)
                .filter(|p| self.game(*p) == kind)
                .collect::<Vec<u32>>();
            let mut kind_games: Vec<AnyGame> = match kind {
                GameKind::DrRustario => self
                    .dr_rustario
                    .games(players.len())?
                    .into_iter()
                    .map(AnyGame::DrRustario)
                    .collect(),
                GameKind::Rustris => self
                    .rustris
                    .games(players.len())
                    .into_iter()
                    .map(AnyGame::Rustris)
                    .collect(),
            };
            for player in players.into_iter().rev() {
                games[player as usize] = kind_games.pop();
            }
        }
        Ok(games.into_iter().map(|g| g.unwrap()).collect())
    }

    fn subtitle(&self) -> String {
        let names = self
            .kinds()
            .iter()
            .map(|k| k.name())
            .collect::<Vec<&str>>()
            .join(" vs. ");
        if self.players == 1 {
            format!("{} single player", names)
        } else {
            format!("{} {}-player vs.", names, self.players)
        }
    }
}

/// Every theme of every game in one list, with each game's slice of it.
struct Themes<'a> {
    all: Vec<Theme<'a>>,
    dr_rustario: Range<usize>,
    rustris: Range<usize>,
}

impl<'a> Themes<'a> {
    fn range(&self, game: GameKind) -> Range<usize> {
        match game {
            GameKind::DrRustario => self.dr_rustario.clone(),
            GameKind::Rustris => self.rustris.clone(),
        }
    }

    fn race(&self) -> Vec<RaceTheme> {
        let mut race = dr_rustario::theme::race_themes(&self.all[self.dr_rustario.clone()]);
        for mut theme in rustris::theme::race_themes(&self.all[self.rustris.clone()]) {
            theme.theme += self.rustris.start;
            race.push(theme);
        }
        race
    }
}

fn main() -> Result<(), String> {
    engine::app_info::init(engine::app_info::AppInfo {
        name: build_info::PKG_NAME,
        version: build_info::PKG_VERSION,
        authors: build_info::PKG_AUTHORS,
    });

    let mut app = App::new(MAX_PLAYERS, include_bytes!("../icon.png"))?;
    let texture_creator = app.canvas().texture_creator();
    let config = app.config();
    let mut all = dr_rustario::theme::all_themes(app.canvas(), &texture_creator, config)?;
    let dr_range = 0..all.len();
    all.extend(rustris::theme::all_themes(app.canvas(), &texture_creator, config)?);
    let themes = Themes {
        rustris: dr_range.end..all.len(),
        dr_rustario: dr_range,
        all,
    };
    let race = themes.race();

    let mut fg_particles = app.particle_render(
        &texture_creator,
        MAX_PARTICLES_PER_PLAYER * MAX_PLAYERS as usize,
        vec![],
    )?;
    let mut bg_particles = app.particle_render(
        &texture_creator,
        MAX_BACKGROUND_PARTICLES,
        themes.all.iter().collect(),
    )?;

    let mut selection = Selection {
        players: 1,
        games: vec![Choice::Game(GameKind::DrRustario); MAX_PLAYERS as usize],
        dr_rustario: Default::default(),
        rustris: Default::default(),
    };
    selection.set_players(1);

    'title: loop {
        let mut items = vec![];
        if MAX_PLAYERS > 1 {
            items.push(MenuItem::select_list(
                PLAYERS,
                (1..=MAX_PLAYERS).map(|i| i.to_string()).collect(),
                selection.players as usize - 1,
            ));
        }
        for player in 0..MAX_PLAYERS {
            items.push(MenuItem::select_list(
                &game_item(player),
                Choice::ALL.iter().map(|c| c.name()).collect(),
                Choice::ALL
                    .iter()
                    .position(|c| *c == selection.choice(player))
                    .unwrap(),
            ));
        }
        items.push(MenuItem::select(HIGH_SCORES));
        items.push(MenuItem::select(START));
        items.push(MenuItem::select(QUIT));

        let exit = app.run_menu(
            items,
            build_info::nice_app_name(),
            None,
            MenuMusic::Title,
            &mut bg_particles,
            &race,
            |name, value| match name {
                PLAYERS => {
                    selection.set_players(value.parse::<u32>().unwrap());
                    None
                }
                HIGH_SCORES => Some(MenuExit::Custom(TitleAction::ViewHighScores)),
                START => Some(MenuExit::Start),
                QUIT => Some(MenuExit::Back),
                _ => {
                    for player in 0..MAX_PLAYERS {
                        if name == game_item(player) {
                            selection.games[player as usize] = Choice::from_name(value).unwrap();
                        }
                    }
                    None
                }
            },
        )?;
        match exit {
            MenuExit::Start => {}
            MenuExit::Back | MenuExit::Quit => break 'title,
            MenuExit::Custom(TitleAction::ViewHighScores) => {
                app.view_high_score(&selection.high_score_key(), &mut bg_particles)?;
                continue 'title;
            }
        }

        'menu: loop {
            let mut items = selection.menu_items();
            items.push(MenuItem::select(START));
            items.push(MenuItem::select(BACK));
            let exit = app.run_menu::<()>(
                items,
                build_info::nice_app_name(),
                Some(selection.subtitle()),
                MenuMusic::Menu,
                &mut bg_particles,
                &race,
                |name, value| match name {
                    START => Some(MenuExit::Start),
                    BACK => Some(MenuExit::Back),
                    _ => {
                        selection.select(name, value);
                        None
                    }
                },
            )?;
            match exit {
                MenuExit::Start => {}
                MenuExit::Back => continue 'title,
                MenuExit::Quit => break 'title,
                MenuExit::Custom(()) => {}
            }

            let games = selection.games()?;
            let settings = selection.settings(&themes);
            let key = settings.high_score_key.clone();
            let next_stage = |player: u32, completed: u32| {
                let choice = selection.choice(player);
                if !choice.is_playlist() {
                    return None;
                }
                let kind = choice.stage(completed);
                let game = selection.stage_game(kind).ok()?;
                Some((game, selection.player_settings(&themes, kind)))
            };
            match app.run_match(
                &themes.all,
                games,
                settings,
                &mut fg_particles,
                &mut bg_particles,
                next_stage,
            )? {
                PostGameAction::NewHighScore(high_score) => {
                    app.new_high_score(&key, high_score, &mut bg_particles)?;
                    continue 'menu;
                }
                PostGameAction::ReturnToMenu => continue 'menu,
                PostGameAction::Quit => break 'title,
            }
        }
    }

    Ok(())
}
