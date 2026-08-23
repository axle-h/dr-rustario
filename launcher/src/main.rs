#![windows_subsystem = "windows"]

mod games;

use crate::games::{AnyGame, GameKind};
use engine::app::{
    App, MatchSettings, MenuExit, MenuMusic, PostGameAction, MAX_BACKGROUND_PARTICLES,
    MAX_PARTICLES_PER_PLAYER,
};
use engine::menu::MenuItem;
use engine::particles::prescribed::RaceTheme;
use engine::render::Theme;

mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));

    pub fn nice_app_name() -> String {
        titlecase::titlecase(&PKG_NAME.replace("-", ". "))
    }
}

#[cfg(not(feature = "retro_handheld"))]
const MAX_PLAYERS: u32 = 2;

#[cfg(feature = "retro_handheld")]
const MAX_PLAYERS: u32 = 1;

const PLAYERS: &str = "players";
const GAME: &str = "game";
const HIGH_SCORES: &str = "high scores";
const START: &str = "start";
const BACK: &str = "back";
const QUIT: &str = "quit";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TitleAction {
    ViewHighScores,
}

/// What the player has picked so far.
struct Selection {
    players: u32,
    game: GameKind,
    dr_rustario: dr_rustario::options::Options,
    rustris: rustris::options::Options,
}

impl Selection {
    fn set_players(&mut self, players: u32) {
        self.players = players;
        self.dr_rustario.set_players(players);
        self.rustris.set_players(players);
    }

    fn menu_items(&self) -> Vec<MenuItem> {
        match self.game {
            GameKind::DrRustario => self.dr_rustario.menu_items(),
            GameKind::Rustris => self.rustris.menu_items(),
        }
    }

    fn select(&mut self, name: &str, value: &str) -> bool {
        match self.game {
            GameKind::DrRustario => self.dr_rustario.select(name, value),
            GameKind::Rustris => self.rustris.select(name, value),
        }
    }

    fn settings(&self) -> MatchSettings {
        match self.game {
            GameKind::DrRustario => self.dr_rustario.settings(),
            GameKind::Rustris => self.rustris.settings(),
        }
    }

    fn games(&self) -> Result<Vec<AnyGame>, String> {
        Ok(match self.game {
            GameKind::DrRustario => self
                .dr_rustario
                .games()?
                .into_iter()
                .map(AnyGame::DrRustario)
                .collect(),
            GameKind::Rustris => self
                .rustris
                .games()
                .into_iter()
                .map(AnyGame::Rustris)
                .collect(),
        })
    }

    fn subtitle(&self) -> String {
        if self.players == 1 {
            format!("{} single player", self.game.name())
        } else {
            format!("{} {}-player vs.", self.game.name(), self.players)
        }
    }
}

/// Every theme of every game, as one list for the particle renderer's race sprites.
struct Themes<'a> {
    dr_rustario: Vec<Theme<'a>>,
    rustris: Vec<Theme<'a>>,
}

impl<'a> Themes<'a> {
    fn all(&self) -> Vec<&Theme<'a>> {
        self.dr_rustario.iter().chain(self.rustris.iter()).collect()
    }

    fn of(&self, game: GameKind) -> &[Theme<'a>] {
        match game {
            GameKind::DrRustario => &self.dr_rustario,
            GameKind::Rustris => &self.rustris,
        }
    }

    fn race(&self) -> Vec<RaceTheme> {
        let mut race = dr_rustario::theme::race_themes(&self.dr_rustario);
        let offset = self.dr_rustario.len();
        for mut theme in rustris::theme::race_themes(&self.rustris) {
            theme.theme += offset;
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
    let themes = Themes {
        dr_rustario: dr_rustario::theme::all_themes(app.canvas(), &texture_creator, config)?,
        rustris: rustris::theme::all_themes(app.canvas(), &texture_creator, config)?,
    };
    let race = themes.race();

    let mut fg_particles = app.particle_render(
        &texture_creator,
        MAX_PARTICLES_PER_PLAYER * MAX_PLAYERS as usize,
        vec![],
    )?;
    let mut bg_particles =
        app.particle_render(&texture_creator, MAX_BACKGROUND_PARTICLES, themes.all())?;

    let mut selection = Selection {
        players: 1,
        game: GameKind::DrRustario,
        dr_rustario: Default::default(),
        rustris: Default::default(),
    };
    selection.set_players(1);

    'title: loop {
        let mut items = vec![
            MenuItem::select_list(
                GAME,
                GameKind::ALL.iter().map(|k| k.name().to_string()).collect(),
                GameKind::ALL
                    .iter()
                    .position(|k| *k == selection.game)
                    .unwrap(),
            ),
            MenuItem::select(HIGH_SCORES),
            MenuItem::select(START),
            MenuItem::select(QUIT),
        ];
        if MAX_PLAYERS > 1 {
            items.insert(
                0,
                MenuItem::select_list(
                    PLAYERS,
                    (1..=MAX_PLAYERS).map(|i| i.to_string()).collect(),
                    selection.players as usize - 1,
                ),
            );
        }
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
                GAME => {
                    selection.game = GameKind::from_name(value).unwrap();
                    None
                }
                HIGH_SCORES => Some(MenuExit::Custom(TitleAction::ViewHighScores)),
                START => Some(MenuExit::Start),
                QUIT => Some(MenuExit::Back),
                _ => None,
            },
        )?;
        match exit {
            MenuExit::Start => {}
            MenuExit::Back | MenuExit::Quit => break 'title,
            MenuExit::Custom(TitleAction::ViewHighScores) => {
                app.view_high_score(&mut bg_particles)?;
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
            match app.run_match(
                themes.of(selection.game),
                games,
                selection.settings(),
                &mut fg_particles,
                &mut bg_particles,
            )? {
                PostGameAction::NewHighScore(high_score) => {
                    app.new_high_score(high_score, &mut bg_particles)?;
                    continue 'menu;
                }
                PostGameAction::ReturnToMenu => continue 'menu,
                PostGameAction::Quit => break 'title,
            }
        }
    }

    Ok(())
}
