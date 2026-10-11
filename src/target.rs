/// What the mouse pointer is over, registered while drawing: an item of a
/// Catfyn screen (`Fin(row, column)`, `usize::MAX` for the header), or
/// nothing of interest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Fin(usize, usize),
    Outside,
}
