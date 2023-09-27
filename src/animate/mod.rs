pub mod destroy;
pub mod dr;
pub mod event;
pub mod game_over;
pub mod hard_drop;
pub mod idle;
pub mod impact;
pub mod lock;
pub mod next_level;
pub mod next_level_interstitial;
pub mod throw;
pub mod victory;
pub mod virus;

use crate::animate::destroy::DestroyAnimation;
use crate::animate::event::{AnimationEvent, AnimationType};
use crate::animate::game_over::GameOverAnimation;
use crate::animate::hard_drop::HardDropAnimation;
use crate::animate::idle::IdleAnimation;
use crate::animate::impact::ImpactAnimation;
use crate::animate::lock::LockAnimation;
use crate::animate::next_level::NextLevelAnimation;
use crate::animate::next_level_interstitial::NextLevelInterstitialAnimation;
use crate::animate::throw::ThrowAnimation;
use crate::animate::victory::VictoryAnimation;
use crate::animate::virus::VirusAnimation;
use crate::theme::Theme;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct PlayerAnimations {
    player: u32,
    idle: IdleAnimation,
    virus: VirusAnimation,
    destroy: DestroyAnimation,
    impact: ImpactAnimation,
    lock: LockAnimation,
    hard_drop: HardDropAnimation,
    throw: ThrowAnimation,
    game_over: GameOverAnimation,
    victory: VictoryAnimation,
    next_level: NextLevelAnimation,
    next_level_interstitial: NextLevelInterstitialAnimation,
}

impl PlayerAnimations {
    pub fn new(player: u32, theme: &Theme) -> Self {
        let meta = theme.animation_meta();
        let idle = IdleAnimation::new(meta.dr_idle_frames, meta.dr_idle_type);
        let virus = VirusAnimation::new(
            meta.red_virus_frames,
            meta.blue_virus_frames,
            meta.yellow_virus_frames,
            meta.virus_type,
        );
        let destroy = DestroyAnimation::new(meta.vitamin_pop_frames, meta.virus_pop_frames);
        let impact = ImpactAnimation::new();
        let lock = LockAnimation::new();
        let hard_drop = HardDropAnimation::new();
        let throw = ThrowAnimation::new(
            meta.throw_start,
            meta.throw_end,
            theme.geometry().block_size(),
            meta.dr_throw_frames,
            meta.dr_throw_type,
        );
        let game_over = GameOverAnimation::new(
            meta.game_over_screen_frames,
            meta.dr_game_over_type,
            meta.dr_game_over_frames,
        );
        let victory = VictoryAnimation::new(meta.dr_victory_frames, meta.dr_victory_type);
        let next_level = NextLevelAnimation::new();
        let next_level_interstitial = NextLevelInterstitialAnimation::new(
            meta.dr_victory_type,
            meta.dr_victory_frames,
            meta.next_level_interstitial_frames,
        );

        Self {
            player,
            idle,
            virus,
            destroy,
            impact,
            lock,
            hard_drop,
            throw,
            game_over,
            victory,
            next_level,
            next_level_interstitial,
        }
    }

    pub fn reset(&mut self) {
        self.idle.reset();
        self.virus.reset();
        self.destroy.reset();
        self.impact.reset();
        self.lock.reset();
        self.hard_drop.reset();
        self.throw.reset();
    }

    pub fn update(&mut self, delta: Duration) -> Vec<AnimationEvent> {
        if delta.is_zero() {
            return vec![];
        }

        let mut events = vec![];
        self.idle.update(delta);
        self.virus.update(delta);
        self.destroy.update(delta);
        self.impact.update(delta);
        self.lock.update(delta);
        self.hard_drop.update(delta);
        if self.throw.update(delta) {
            events.push(AnimationEvent::Finished {
                animation: AnimationType::Throw,
                player: self.player,
            });
        }
        self.game_over.update(delta);
        self.victory.update(delta);
        self.next_level.update(delta);
        self.next_level_interstitial.update(delta);
        events
    }

    pub fn is_animating(&self) -> bool {
        self.destroy.state().is_some()
            || self.lock.state().is_some()
            || self.hard_drop.state().is_some()
            || self.throw.state().is_some()
            || self.game_over.state().is_some()
            || self.victory.state().is_some()
            || self.next_level.state().is_some()
            || self.next_level_interstitial.state().is_some()
    }

    pub fn idle(&self) -> &IdleAnimation {
        &self.idle
    }

    pub fn virus(&self) -> &VirusAnimation {
        &self.virus
    }

    pub fn destroy(&self) -> &DestroyAnimation {
        &self.destroy
    }

    pub fn destroy_mut(&mut self) -> &mut DestroyAnimation {
        &mut self.destroy
    }

    pub fn impact(&self) -> &ImpactAnimation {
        &self.impact
    }

    pub fn impact_mut(&mut self) -> &mut ImpactAnimation {
        &mut self.impact
    }

    pub fn lock(&self) -> &LockAnimation {
        &self.lock
    }

    pub fn lock_mut(&mut self) -> &mut LockAnimation {
        &mut self.lock
    }

    pub fn hard_drop(&self) -> &HardDropAnimation {
        &self.hard_drop
    }

    pub fn hard_drop_mut(&mut self) -> &mut HardDropAnimation {
        &mut self.hard_drop
    }

    pub fn throw(&self) -> &ThrowAnimation {
        &self.throw
    }

    pub fn throw_mut(&mut self) -> &mut ThrowAnimation {
        &mut self.throw
    }

    pub fn game_over(&self) -> &GameOverAnimation {
        &self.game_over
    }

    pub fn game_over_mut(&mut self) -> &mut GameOverAnimation {
        &mut self.game_over
    }

    pub fn victory(&self) -> &VictoryAnimation {
        &self.victory
    }

    pub fn victory_mut(&mut self) -> &mut VictoryAnimation {
        &mut self.victory
    }

    pub fn next_level(&self) -> &NextLevelAnimation {
        &self.next_level
    }

    pub fn next_level_mut(&mut self) -> &mut NextLevelAnimation {
        &mut self.next_level
    }

    pub fn next_level_interstitial(&self) -> &NextLevelInterstitialAnimation {
        &self.next_level_interstitial
    }

    pub fn next_level_interstitial_mut(&mut self) -> &mut NextLevelInterstitialAnimation {
        &mut self.next_level_interstitial
    }
}
