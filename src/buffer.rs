use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::{CursorMove, CursorMove::*};

pub type Position = (i64, i64);

pub struct Buffer
{
    pub src_path: Option<PathBuf>,
    pub content: Vec<String>,
    pub cursor: Position,
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
        content: content.split('\n').map(|r| String::from(r)).collect(),
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
        line.insert(cx as usize, c);
        move_cursor(buffer, Relative(1, 0));
        Ok(())
    } else {
        Err("line not available")
    }
}

pub fn insert_newline(buffer: &mut Buffer) -> Result<(), &'static str>
{
    let (cx, cy) = buffer.cursor;
    let line = buffer.content.get_mut(cy as usize);
    if line.is_none() {
        return Err("line not available");
    }
    let line = line.unwrap();

    {
        let (left, right) = {
            let (l, r) = line.split_at(cx as usize);
            (l.to_string(), r.to_string())
        };
        *line = left;
        buffer.content.insert((cy + 1) as usize, right);
    }

    move_cursor(buffer, Absolute(0, cy + 1));

    Ok(())
}

pub fn remove(buffer: &mut Buffer) -> Result<(), &'static str>
{
    let (cx, cy) = buffer.cursor;
    let (nx, ny) = (cx - 1, cy - 1);

    if let Some(line) = buffer.content.get_mut(cy as usize) {
        // TODO: that is disgusting
        let len = line.len() as i64;
        if 0 <= nx && nx < len {
            line.remove(nx as usize);
            move_cursor(buffer, Relative(-1, 0));
        }
        if nx < 0 && 0 <= ny && 1 < buffer.content.len() {
            let removed = buffer.content.remove(cy as usize);
            move_cursor(buffer, EndOfRow(ny));
            if len != 0 {
                buffer
                    .content
                    .get_mut(ny as usize)
                    .expect("line for appending not available")
                    .push_str(&removed);
            }
        }
        Ok(())
    } else {
        Err("line not available")
    }
}

pub fn get_row_at(buffer: &Buffer, line: usize) -> Option<&str>
{
    buffer.content.get(line).and_then(|c| Some(c.as_ref()))
}

pub fn move_cursor(buffer: &mut Buffer, mv: CursorMove)
{
    let (x, y) = match mv {
        Absolute(x, y) => (x, y),
        EndOfRow(y) => (0, y),
        CurrentRow(x) => (x, buffer.cursor.1),
        Relative(rx, ry) => {
            let (cx, cy) = buffer.cursor;
            (cx + rx, cy + ry)
        }
    };
    if let Some(line) = buffer.content.get(y as usize) {
        buffer.cursor.1 = y;
        let len = line.len() as i64;

        match mv {
            EndOfRow(_) => buffer.cursor.0 = len,
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
