pub mod buffer;

pub mod input
{
    pub enum CursorMove
    {
        Absolute(i64, i64),
        Relative(i64, i64),
        EndOfRow(i64),
        CurrentRow(i64),
    }
}

pub mod plugin
{
    pub type NameCallback = unsafe extern "C" fn() -> &'static str;
    pub type CommandsCallback = unsafe extern "C" fn() -> Vec<String>;
    pub type DispatchCallback =
        unsafe extern "C" fn(&mut crate::buffer::Buffer, &str) -> DispatchResult;
    pub type DispatchResult = Result<(), String>;
    pub type UnloadCallback = unsafe extern "C" fn();
}

#[cfg(test)]
mod tests
{
    use super::{buffer::*, input::CursorMove};

    fn get_default_buffer() -> Buffer
    {
        let text = r#"First line
Second line here
"#;

        Buffer {
            src_path: None,
            content: text.split('\n').map(String::from).collect(),
            cursor: (0, 0),
        }
    }

    fn insert_str(buffer: &mut Buffer, ins: &str) -> bool
    {
        for c in ins.chars() {
            if insert(buffer, c).is_err() {
                return false;
            }
        }
        true
    }

    #[test]
    pub fn buffer_editing()
    {
        let mut buf = get_default_buffer();
        assert_eq!(buf.cursor, (0, 0));

        // insert front of line
        assert!(insert_str(&mut buf, "very "));
        assert_eq!(get_row_at(&mut buf, 0), Some("very First line"));

        // append to line
        move_cursor(&mut buf, CursorMove::EndOfRow(0));
        assert!(insert_str(&mut buf, " appended"));
        assert_eq!(get_row_at(&mut buf, 0), Some("very First line appended"));

        // second line unchanged
        assert_eq!(get_row_at(&mut buf, 1), Some("Second line here"));

        // set cursor on word boundary of `line`
        move_cursor(&mut buf, CursorMove::Absolute(11, 0));
        assert!(insert_newline(&mut buf).is_ok());
        assert_eq!(get_row_at(&mut buf, 0), Some("very First "));
        assert_eq!(get_row_at(&mut buf, 1), Some("line appended"));
        // cursor must be set on second half of split
        assert_eq!(buf.cursor, (0, 1));
        // original second line unchanged
        assert_eq!(get_row_at(&mut buf, 2), Some("Second line here"));

        // append new line to text
        move_cursor(&mut buf, CursorMove::EndOfRow(2));
        assert!(insert_newline(&mut buf).is_ok());
        assert!(insert_str(&mut buf, "added third line"));
        assert_eq!(get_row_at(&mut buf, 3), Some("added third line"));
        assert_eq!(buf.cursor, (16, 3));

        // join two independent lines
        move_cursor(&mut buf, CursorMove::CurrentRow(0));
        assert!(remove(&mut buf).is_ok());
        assert_eq!(
            get_row_at(&mut buf, 2),
            Some("Second line hereadded third line")
        );

        move_cursor(&mut buf, CursorMove::Absolute(0, 0));
        // removing from first position in file not possible
        assert!(remove(&mut buf).is_err());
    }

    #[test]
    pub fn buffer_operations()
    {
        let mut buf = get_default_buffer();

        // retrieving lines from buffer
        assert_eq!(get_row_at(&mut buf, 0), Some("First line"));
        assert_eq!(get_row_at(&mut buf, 1), Some("Second line here"));
        assert_eq!(get_row_at(&mut buf, 99), None);
    }

    #[test]
    pub fn buffer_movement()
    {
        let mut buf = get_default_buffer();

        move_cursor(&mut buf, CursorMove::Relative(1, 0));
        assert_eq!(buf.cursor, (1, 0));

        // line -1 does not exist -> no move
        move_cursor(&mut buf, CursorMove::Relative(-2, 0));
        assert_eq!(buf.cursor, (1, 0));

        move_cursor(&mut buf, CursorMove::Relative(1, 1));
        assert_eq!(buf.cursor, (2, 1));

        // line -1 does not exist -> no move
        move_cursor(&mut buf, CursorMove::Relative(0, -2));
        assert_eq!(buf.cursor, (2, 1));

        move_cursor(&mut buf, CursorMove::EndOfRow(0));
        assert_eq!(buf.cursor, (10, 0));

        move_cursor(&mut buf, CursorMove::EndOfRow(1));
        assert_eq!(buf.cursor, (16, 1));

        // if current line is longer than target line:
        // set cursor_x to target line len
        move_cursor(&mut buf, CursorMove::Relative(0, -1));
        assert_eq!(buf.cursor, (10, 0));

        // line 99 does not exist -> no move
        move_cursor(&mut buf, CursorMove::EndOfRow(99));
        assert_eq!(buf.cursor, (10, 0));

        // line 99 does not exist -> no move
        move_cursor(&mut buf, CursorMove::Absolute(0, 99));
        assert_eq!(buf.cursor, (10, 0));

        move_cursor(&mut buf, CursorMove::Absolute(0, 0));
        assert_eq!(buf.cursor, (0, 0));
    }

}
