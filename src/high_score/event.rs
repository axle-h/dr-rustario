#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HighScoreEntryEvent {
    CursorRight,
    CursorLeft,
    ChangeChar,
    Finished,
}