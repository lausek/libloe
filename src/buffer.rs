use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::input::{CursorMove, CursorMove::*};

pub type Position = (i64, i64);

pub struct Buffer
{
    pub src_path: Option<PathBuf>,
    pub content: Vec<String>,
    pub cursor: Position,
}

// normalizes the cursor position to a `String` internal codepoint
fn get_char_index(line: &str, pos: usize) -> usize
{
    line.char_indices()
        .nth(pos)
        .and_then(|(idx, _)| Some(idx))
        .unwrap_or_else(|| line.len())
}

// &str.len() doesn't recognise multibyte chars
fn get_row_len(line: &str) -> usize
{
    line.chars().count()
}

pub fn get_row_at(buffer: &Buffer, line: usize) -> Option<&str>
{
    buffer.content.get(line).and_then(|c| Some(c.as_ref()))
}

pub fn create(path: &str) -> Result<Buffer, std::io::Error>
{
    let mut pathbuf = PathBuf::new();
    // TODO: normalize path here
    pathbuf.push(path);

    let buffer = Buffer {
        cursor: (0, 0),
        src_path: Some(pathbuf),
        content: vec![String::new()],
    };
    Ok(buffer)
}

pub fn load(path: &str) -> Result<Buffer, std::io::Error>
{
    let mut pathbuf = PathBuf::new();
    let path = std::fs::canonicalize(path)?;
    pathbuf.push(path);

    let content = std::fs::read_to_string(pathbuf.as_path())?;

    let buffer = Buffer {
        cursor: (0, 0),
        src_path: Some(pathbuf),
        content: content.split('\n').map(String::from).collect(),
    };
    Ok(buffer)
}

pub fn write(buffer: &Buffer, path: &PathBuf) -> Result<(), std::io::Error>
{
    File::create(path).and_then(|mut file| {
        let mut puf = vec![];
        for line in &buffer.content {
            puf.extend_from_slice(line.as_bytes());
            puf.extend_from_slice(b"\n");
        }
        file.write_all(&puf)?;
        file.sync_all()
    })
}

pub fn insert(buffer: &mut Buffer, c: char) -> Result<(), &'static str>
{
    let (cx, cy) = buffer.cursor;
    if let Some(line) = buffer.content.get_mut(cy as usize) {
        let idx = get_char_index(line, cx as usize);
        line.insert(idx, c);
        move_cursor(buffer, Relative(1, 0));
        Ok(())
    } else {
        Err("line not available")
    }
}

pub fn insert_newline(buffer: &mut Buffer) -> Result<(), &'static str>
{
    let (cx, cy) = buffer.cursor;
    if let Some(line) = buffer.content.get_mut(cy as usize) {
        let idx = get_char_index(line, cx as usize);
        let (left, right) = {
            let (l, r) = line.split_at(idx as usize);
            (l.to_string(), r.to_string())
        };
        *line = left;
        buffer.content.insert((cy + 1) as usize, right);

        move_cursor(buffer, Absolute(0, cy + 1));

        Ok(())
    } else {
        Err("line not available")
    }
}

pub fn remove(buffer: &mut Buffer) -> Result<(), &'static str>
{
    let (cx, cy) = buffer.cursor;
    // position on which the cursor will be set after removing
    let (next_x, next_y) = (cx - 1, cy - 1);

    if let Some(line) = buffer.content.get_mut(cy as usize) {
        // TODO: that is disgusting
        let len = get_row_len(line) as i64;
        // if next cursor pos is still in line's range: no linebreak needed
        if 0 <= next_x && next_x < len {
            let idx = get_char_index(line, next_x as usize);
            if idx == len as usize {
                line.pop().unwrap();
            } else {
                line.remove(idx);
            }
            move_cursor(buffer, Relative(-1, 0));
            return Ok(());
        }
    } else {
        return Err("line not available");
    }
    // if next cursor pos is not in line's range
    if next_x < 0 && 0 <= next_y && 1 < buffer.content.len() {
        //let idx = get_char_index(line, cy as usize);
        // remove whole line and append to next one
        let removed = buffer.content.remove(cy as usize);
        move_cursor(buffer, AfterRow(next_y));
        if !removed.is_empty() {
            buffer
                .content
                .get_mut(next_y as usize)
                .expect("line for appending not available")
                .push_str(&removed);
        }
        return Ok(());
    }
    Err("move is invalid")
}

// retrun false or true wether the move has been executed
pub fn move_cursor(buffer: &mut Buffer, mv: CursorMove)
{
    let (x, y) = match mv {
        Absolute(x, y) => (x, y),
        EndOfRow(y) | AfterRow(y) => (0, y),
        CurrentRow(x) => (x, buffer.cursor.1),
        Relative(rx, ry) => {
            let (cx, cy) = buffer.cursor;
            (cx + rx, cy + ry)
        }
    };
    if let Some(line) = buffer.content.get(y as usize) {
        buffer.cursor.1 = y;
        let len = get_row_len(line) as i64;

        match mv {
            EndOfRow(_) => buffer.cursor.0 = len - 1,
            AfterRow(_) => buffer.cursor.0 = len,
            _ => {
                if 0 <= x {
                    buffer.cursor.0 = x;
                    if len < buffer.cursor.0 {
                        buffer.cursor.0 = len;
                    }
                }
            }
        }
    }
}
