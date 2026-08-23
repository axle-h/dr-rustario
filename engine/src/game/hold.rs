/// The held piece and whether hold is blocked until the current piece locks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoldState<T: Copy> {
    pub piece: T,
    pub locked: bool,
}

impl<T: Copy> HoldState<T> {
    pub fn locked(piece: T) -> Self {
        Self {
            piece,
            locked: true,
        }
    }

    pub fn is_locked(hold: &Option<Self>) -> bool {
        matches!(hold, Some(HoldState { locked: true, .. }))
    }

    pub fn unlock(hold: &mut Option<Self>) {
        if let Some(hold) = hold.as_mut() {
            hold.locked = false;
        }
    }
}
