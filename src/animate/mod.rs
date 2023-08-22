pub mod virus;
pub mod destroy;
pub mod impact;
pub mod lock;
pub mod hard_drop;
pub mod spawn;
pub mod game_over;
pub mod victory;
pub mod next_level;

use std::time::Duration;
use crate::animate::destroy::DestroyAnimation;
use crate::animate::game_over::GameOverAnimation;
use crate::animate::hard_drop::HardDropAnimation;
use crate::animate::impact::ImpactAnimation;
use crate::animate::lock::LockAnimation;
use crate::animate::next_level::NextLevelAnimation;
use crate::animate::spawn::SpawnAnimation;
use crate::animate::victory::VictoryAnimation;
use crate::animate::virus::VirusAnimation;
use crate::theme::Theme;

#[derive(Clone, Debug)]
pub struct PlayerAnimations {
    virus: VirusAnimation,
    destroy: DestroyAnimation,
    impact: ImpactAnimation,
    lock: LockAnimation,
    hard_drop: HardDropAnimation,
    spawn: SpawnAnimation,
    game_over: GameOverAnimation,
    victory: VictoryAnimation,
    next_level: NextLevelAnimation
}

impl PlayerAnimations {
    pub fn new(theme: &Theme) -> Self {
        let meta = theme.animation_meta();
        let virus = VirusAnimation::new(meta.virus_frames);
        let destroy = DestroyAnimation::new(meta.vitamin_pop_frames);
        let impact = ImpactAnimation::new();
        let lock = LockAnimation::new();
        let hard_drop = HardDropAnimation::new();
        let spawn = SpawnAnimation::new(
            meta.throw_start,
            meta.throw_end,
            theme.geometry().block_size(),
            meta.dr_throw_frames
        );
        let game_over = GameOverAnimation::new(meta.game_over_screen_frames);
        let victory = VictoryAnimation::new(meta.dr_victory_frames);
        let next_level = NextLevelAnimation::new(meta.dr_wait_frames, meta.next_level_interstitial_frames);
        Self { virus, destroy, impact, lock, hard_drop, spawn, game_over, victory, next_level }
    }

    pub fn reset(&mut self) {
        self.virus.reset();
        self.destroy.reset();
        self.impact.reset();
        self.lock.reset();
        self.hard_drop.reset();
        self.spawn.reset();
    }

    pub fn update(&mut self, delta: Duration) {
        if delta.is_zero() {
            return;
        }
        self.virus.update(delta);
        self.destroy.update(delta);
        self.impact.update(delta);
        self.lock.update(delta);
        self.hard_drop.update(delta);
        self.spawn.update(delta);
        self.game_over.update(delta);
        self.victory.update(delta);
        self.next_level.update(delta);
    }

    pub fn is_animating(&self) -> bool {
        self.destroy.state().is_some()
            || self.lock.state().is_some()
            || self.hard_drop.state().is_some()
            || self.spawn.state().is_some()
            || self.game_over.state().is_some()
            || self.victory.state().is_some()
            || self.next_level.state().is_some()
    }

    pub fn maybe_dismiss(&mut self) -> bool {
        self.next_level.maybe_dismiss_interstitial()
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

    pub fn spawn(&self) -> &SpawnAnimation {
        &self.spawn
    }

    pub fn spawn_mut(&mut self) -> &mut SpawnAnimation {
        &mut self.spawn
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

}