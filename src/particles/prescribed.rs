
use crate::particles::color::ParticleColor;
use crate::particles::geometry::Vec2D;
use crate::particles::meta::ParticleSprite;
use crate::particles::particle::ParticleWave;
use crate::particles::quantity::ProbabilityTable;
use crate::particles::scale::Scale;
use crate::particles::source::{
    AggregateParticleSource, ParticleModulation, ParticleProperties, ParticleSource,
    RandomParticleSource,
};
use crate::themes::ThemeContext;
use crate::theme::particle::sprites::SRC_BLOCK_SIZE as MODERN_BLOCK_SIZE;
use crate::theme::nes::BLOCK_SIZE as NES_BLOCK_SIZE;
use crate::theme::snes::BLOCK_SIZE as SNES_BLOCK_SIZE;
use crate::theme::n64::BLOCK_SIZE as N64_BLOCK_SIZE;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use std::time::Duration;
use crate::game::event::ColoredBlock;
use crate::game::geometry::BottlePoint;
use crate::game::pill::{Garbage, VirusColor, Vitamins};
use crate::theme::all::AllThemeMeta;
use crate::theme::{AnimationMeta, ThemeName};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PlayerParticleTarget {
    Vitamins(Vitamins),
    Blocks(Vec<BottlePoint>),
    Garbage(Vec<Garbage>),
    MaskedBlocks(Vec<ColoredBlock>),
    Bottle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrescribedParticles {
    FadeInLatticeBurstAndFall { fade_in: Duration, color: Color },
    LightBurstUpAndOut { color: Color },
    BurstUp { color: Color },
    BurstDown { color: Color },
    PerimeterBurst { color: Color },
    PerimeterSpray { color: Color },
}

impl PrescribedParticles {
    pub fn into_targeted(
        self,
        player: u32,
        target: PlayerParticleTarget,
    ) -> PlayerTargetedParticles {
        PlayerTargetedParticles {
            player,
            target,
            particles: self,
        }
    }

    pub fn into_lattice_source<I: Iterator<Item = Point>>(self, scale: &Scale, lattice: I) -> Box<dyn ParticleSource> {
        match self {
            PrescribedParticles::FadeInLatticeBurstAndFall { fade_in, color } => {
                RandomParticleSource::new(
                    scale.build_lattice(lattice),
                    ParticleModulation::Cascade,
                )
                    .with_static_properties(
                        ParticleSprite::Circle05,
                        ParticleColor::from_sdl(color),
                        1.0,
                        0.0,
                    )
                    .with_velocity((Vec2D::new(0.0, -0.4), Vec2D::new(0.1, 0.1)))
                    .with_acceleration(Vec2D::new(0.0, 1.5)) // gravity
                    .with_anchor(fade_in)
                    .with_fade_in(fade_in)
                    .with_alpha((0.9, 0.1))
                    .into_box()
            }
            _ => unreachable!()
        }
    }

    pub fn into_source(self, scale: &Scale, rects: &[Rect]) -> Box<dyn ParticleSource> {
        match self {
            PrescribedParticles::FadeInLatticeBurstAndFall { fade_in, color } => {
                RandomParticleSource::new(
                    scale.rect_lattice_source(rects),
                    ParticleModulation::Cascade,
                )
                .with_static_properties(
                    ParticleSprite::Circle05,
                    ParticleColor::from_sdl(color),
                    1.0,
                    0.0,
                )
                .with_velocity((Vec2D::new(0.0, -0.4), Vec2D::new(0.1, 0.1)))
                .with_acceleration(Vec2D::new(0.0, 1.5)) // gravity
                .with_anchor(fade_in)
                .with_fade_in(fade_in)
                .with_alpha((0.9, 0.1))
                .into_box()
            }
            PrescribedParticles::LightBurstUpAndOut { color } => RandomParticleSource::burst(
                scale.rect_lattice_source(rects),
                ParticleSprite::Circle05,
                ParticleColor::from_sdl(color),
                (Vec2D::new(0.0, -0.1), Vec2D::new(0.2, 0.2)),
                (1.0, 0.1),
                (0.4, 0.1),
            )
            .into_box(),
            PrescribedParticles::BurstUp { color } => RandomParticleSource::burst(
                scale.rect_lattice_source(rects),
                ParticleSprite::Circle05,
                ParticleColor::from_sdl(color),
                (Vec2D::new(0.0, -0.2), Vec2D::new(0.05, 0.1)),
                (1.0, 0.1),
                (0.7, 0.3),
            )
            .into_box(),
            PrescribedParticles::BurstDown { color } => RandomParticleSource::burst(
                scale.rect_lattice_source(rects),
                ParticleSprite::Circle05,
                ParticleColor::from_sdl(color),
                (Vec2D::new(0.0, 0.2), Vec2D::new(0.1, 0.1)),
                (1.0, 0.1),
                (0.7, 0.3),
            )
            .into_box(),
            PrescribedParticles::PerimeterBurst { color } => {
                let color = ParticleColor::from_sdl(color);
                let sources = rects
                    .iter()
                    .flat_map(|r| perimeter_sources(scale, *r, color))
                    .collect();
                AggregateParticleSource::new(sources).into_box()
            }
            PrescribedParticles::PerimeterSpray { color } => {
                let color = ParticleColor::from_sdl(color);
                let sources = rects
                    .iter()
                    .flat_map(|r| perimeter_sources(scale, *r, color))
                    .map(|s| {
                        s.with_modulation(ParticleModulation::Constant {
                            count: u32::MAX,
                            step: Duration::from_millis(750),
                        })
                    })
                    .collect();
                AggregateParticleSource::new(sources).into_box()
            }
        }
    }
}

pub fn prescribed_fireworks(window: Rect, scale: &Scale) -> Box<dyn ParticleSource> {
    let modulation = ParticleModulation::Constant {
        count: 100,
        step: Duration::from_millis(500),
    };
    let buffer = window.height() / 5;
    let rect = Rect::from_center(
        window.center(),
        window.width() - buffer,
        window.height() - buffer,
    );
    RandomParticleSource::new(scale.random_rect_source(rect), modulation)
        .with_static_properties(
            ParticleSprite::Circle05,
            (
                ParticleColor::rgb(0.5, 0.5, 0.5),
                ParticleColor::rgb(0.5, 0.5, 0.5),
            ),
            1.0,
            0.0,
        )
        .with_velocity((Vec2D::new(0.0, -0.05), Vec2D::new(0.15, 0.15)))
        .with_fade_out((1.5, 0.5))
        .with_acceleration(Vec2D::new(0.0, 0.1)) // gravity
        .with_alpha((0.9, 0.1))
        .into_box()
}

pub fn prescribed_vitamin_race(window: Rect, scale: &Scale, theme_meta: AllThemeMeta) -> Box<dyn ParticleSource> {
    let modulation = ParticleModulation::Constant {
        count: 1,
        step: Duration::from_millis(1000),
    };
    let buffer_y = window.height() / 10;
    let rect = Rect::new(
        window.left() - 50,
        window.top() + buffer_y as i32,
        50,
        window.height() - 2 * buffer_y,
    );
    let nes_block_scale = MODERN_BLOCK_SIZE as f64 / NES_BLOCK_SIZE as f64 / 2.0;
    let nes_scale = (nes_block_scale, nes_block_scale / 5.0);
    let snes_block_scale = MODERN_BLOCK_SIZE as f64 / SNES_BLOCK_SIZE as f64 / 2.0;
    let snes_scale = (snes_block_scale, snes_block_scale / 5.0);
    let n64_block_scale = MODERN_BLOCK_SIZE as f64 / N64_BLOCK_SIZE as f64 / 2.0;
    let n64_scale = (n64_block_scale, n64_block_scale / 5.0);
    let modern_scale = (1.0, 0.2);
    let rotation = (0.0, 30.0);
    let p_virus = 1.0 / 3.0;
    RandomParticleSource::new(scale.rect_source(rect), modulation)
        .with_properties(
            ProbabilityTable::new()
                .with_1(
                    ParticleProperties::simple(&ParticleSprite::MODERN_PILLS, modern_scale)
                        .angular_velocity(rotation),
                )
                .with(
                    ParticleProperties::simple(
                        &[
                            ParticleSprite::Virus(ThemeName::Particle, VirusColor::Red, theme_meta.particle.virus_particle_animation(VirusColor::Red)),
                            ParticleSprite::Virus(ThemeName::Particle, VirusColor::Blue, theme_meta.particle.virus_particle_animation(VirusColor::Blue)),
                            ParticleSprite::Virus(ThemeName::Particle, VirusColor::Yellow, theme_meta.particle.virus_particle_animation(VirusColor::Yellow))
                        ],
                        modern_scale
                    ).angular_velocity(rotation),
                    p_virus
                )

                .with_1(
                    ParticleProperties::simple(&ParticleSprite::NES_PILLS, nes_scale)
                        .angular_velocity(rotation),
                )
                .with(
                    ParticleProperties::simple(
                        &[
                            ParticleSprite::Virus(ThemeName::Nes, VirusColor::Red, theme_meta.nes.virus_particle_animation(VirusColor::Red)),
                            ParticleSprite::Virus(ThemeName::Nes, VirusColor::Blue, theme_meta.nes.virus_particle_animation(VirusColor::Blue)),
                            ParticleSprite::Virus(ThemeName::Nes, VirusColor::Yellow, theme_meta.nes.virus_particle_animation(VirusColor::Yellow))
                        ],
                        nes_scale
                    ).angular_velocity(rotation),
                    p_virus
                )

                .with_1(
                    ParticleProperties::simple(&ParticleSprite::SNES_PILLS, snes_scale)
                        .angular_velocity(rotation),
                )
                .with(
                    ParticleProperties::simple(
                        &[
                            ParticleSprite::Virus(ThemeName::Snes, VirusColor::Red, theme_meta.snes.virus_particle_animation(VirusColor::Red)),
                            ParticleSprite::Virus(ThemeName::Snes, VirusColor::Blue, theme_meta.snes.virus_particle_animation(VirusColor::Blue)),
                            ParticleSprite::Virus(ThemeName::Snes, VirusColor::Yellow, theme_meta.snes.virus_particle_animation(VirusColor::Yellow))
                        ],
                        snes_scale
                    ).angular_velocity(rotation),
                    p_virus
                )

                .with_1(
                    ParticleProperties::simple(&ParticleSprite::N64_PILLS, n64_scale)
                        .angular_velocity(rotation),
                )
                .with(
                    ParticleProperties::simple(
                        &[
                            ParticleSprite::Virus(ThemeName::N64, VirusColor::Red, theme_meta.n64.virus_particle_animation(VirusColor::Red)),
                            ParticleSprite::Virus(ThemeName::N64, VirusColor::Blue, theme_meta.n64.virus_particle_animation(VirusColor::Blue)),
                            ParticleSprite::Virus(ThemeName::N64, VirusColor::Yellow, theme_meta.n64.virus_particle_animation(VirusColor::Yellow))
                        ],
                        n64_scale
                    ).angular_velocity(rotation),
                    p_virus
                ),
        )
        .with_velocity((Vec2D::new(0.2, 0.0), Vec2D::new(0.05, 0.02)))
        .with_alpha((0.9, 0.1))
        .into_box()
}

pub fn prescribed_orbit(window: Rect, scale: &Scale) -> Box<dyn ParticleSource> {
    const V: f64 = 0.05;
    let [top_left, top_right, bottom_right, bottom_left] = rect_quadrants(window);
    let sources = vec![
        orbit_source(scale, top_left, (V, -V)),
        orbit_source(scale, top_right, (V, V)),
        orbit_source(scale, bottom_right, (-V, V)),
        orbit_source(scale, bottom_left, (-V, -V)),
    ];
    AggregateParticleSource::new(sources).into_box()
}

fn orbit_source<V: Into<Vec2D>>(scale: &Scale, rect: Rect, velocity: V) -> RandomParticleSource {
    let modulation = ParticleModulation::Constant {
        count: 10,
        step: Duration::from_millis(1000),
    };
    let velocity = velocity.into();
    RandomParticleSource::new(scale.rect_source(rect), modulation)
        .with_properties(
            ProbabilityTable::new()
                .with(
                    ParticleProperties::simple(&[ParticleSprite::Circle05], (1.0, 0.3)),
                    0.8,
                )
                .with(
                    ParticleProperties::new(
                        &ParticleSprite::HOLLOW_CIRCLES,
                        (
                            ParticleColor::rgb(0.6, 0.6, 0.8),
                            ParticleColor::rgb(0.1, 0.1, 0.1),
                        ),
                        (1.5, 0.4),
                        0.0,
                    ),
                    0.1,
                )
                .with(
                    ParticleProperties::new(
                        &ParticleSprite::STARS,
                        (
                            ParticleColor::rgb(0.8, 0.6, 0.6),
                            ParticleColor::rgb(0.1, 0.1, 0.1),
                        ),
                        (1.6, 0.4),
                        0.0,
                    ),
                    0.1,
                ),
        )
        .with_fade_in(Duration::from_millis(500))
        .with_fade_out((10.0, 2.5))
        .with_pulse((ParticleWave::new(0.03, 8.0), ParticleWave::new(0.01, 1.0)))
        .with_velocity((velocity, velocity * 0.5))
        .with_alpha((0.9, 0.1))
        .with_orbit((0.5, 0.5))
}

fn rect_quadrants(rect: Rect) -> [Rect; 4] {
    fn quad(point: Point, rect: Rect) -> Rect {
        Rect::new(point.x(), point.y(), rect.width() / 2, rect.height() / 2)
    }
    [
        quad(rect.top_left(), rect),                            // top left
        quad(Point::new(rect.center().x(), rect.top()), rect),  // top right
        quad(rect.center(), rect),                              // bottom right
        quad(Point::new(rect.left(), rect.center().y()), rect), // bottom left
    ]
}

fn perimeter_sources(scale: &Scale, rect: Rect, color: ParticleColor) -> [RandomParticleSource; 4] {
    const V: f64 = 0.2;
    const FADE_OUT: (f64, f64) = (1.0, 0.1);
    const ALPHA: (f64, f64) = (0.7, 0.3);
    const SPRITE: ParticleSprite = ParticleSprite::Circle05;
    let [top, right, bottom, left] = scale.perimeter_lattice_sources(rect);
    [
        RandomParticleSource::burst(
            top,
            SPRITE,
            color,
            (Vec2D::new(0.0, -V), Vec2D::new(0.2, 0.1)),
            FADE_OUT,
            ALPHA,
        ),
        RandomParticleSource::burst(
            right,
            SPRITE,
            color,
            (Vec2D::new(V, 0.0), Vec2D::new(0.1, 0.2)),
            FADE_OUT,
            ALPHA,
        ),
        RandomParticleSource::burst(
            bottom,
            SPRITE,
            color,
            (Vec2D::new(0.0, V), Vec2D::new(0.2, 0.1)),
            FADE_OUT,
            ALPHA,
        ),
        RandomParticleSource::burst(
            left,
            SPRITE,
            color,
            (Vec2D::new(-V, 0.0), Vec2D::new(0.1, 0.2)),
            FADE_OUT,
            ALPHA,
        ),
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerTargetedParticles {
    player: u32,
    target: PlayerParticleTarget,
    particles: PrescribedParticles,
}

impl PlayerTargetedParticles {
    pub fn into_source(
        self,
        themes: &ThemeContext,
        particle_scale: &Scale,
    ) -> Box<dyn ParticleSource> {
        let target_rects = match self.target {
            PlayerParticleTarget::Bottle => vec![themes.player_bottle_snip(self.player)],
            PlayerParticleTarget::Vitamins(vitamins) => {
                themes.player_vitamin_snips(self.player, vitamins).to_vec()
            }
            PlayerParticleTarget::Blocks(blocks) => {
                themes.player_block_snips(self.player, blocks)
            }
            PlayerParticleTarget::MaskedBlocks(blocks) => {
                let points = themes.player_block_snips_masked(self.player, blocks, 5);
                return self.particles.into_lattice_source(particle_scale, points.into_iter())
            }
            PlayerParticleTarget::Garbage(garbage) => {
                themes.player_block_snips(self.player, garbage.into_iter().map(|g| g.position).collect())
            }
        };

        self.particles
            .into_source(particle_scale, target_rects.as_slice())
    }
}
