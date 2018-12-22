pub mod buffer;

pub enum CursorMove
{
    Absolute(i64, i64),
    Relative(i64, i64),
    EndOfRow(i64),
    CurrentRow(i64),
}
