use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::{CursorMove, CursorMove::*};

pub type Position = (i64, i64);

pub struct Buffer
{
    src_path: Option<PathBuf>,
    content: Vec<String>,
    cursor: Position,
}

impl Buffer
{
    pub fn create(path: &str) -> Result<Self, std::io::Error>
    {
        let mut pathbuf = PathBuf::new();
        // TODO: normalize path here
        pathbuf.push(path);

        let buffer = Self {
            cursor: (0, 0),
            src_path: Some(pathbuf),
            content: vec![String::new()],
        };
        Ok(buffer)
    }

    pub fn load(path: &str) -> Result<Self, std::io::Error>
    {
        let mut pathbuf = PathBuf::new();
        let path = std::fs::canonicalize(path)?;
        pathbuf.push(path);

        let content = std::fs::read_to_string(pathbuf.as_path())?;

        let buffer = Self {
            cursor: (0, 0),
            src_path: Some(pathbuf),
            content: content.split('\n').map(|r| String::from(r)).collect(),
        };
        Ok(buffer)
    }

    pub fn write(&self, path: &PathBuf) -> Result<(), std::io::Error>
    {
        File::create(path).and_then(|mut file| {
            let mut buffer = vec![];
            for line in &self.content {
                buffer.extend_from_slice(line.as_bytes());
                buffer.extend_from_slice(b"\n");
            }
            file.write_all(&buffer)?;
            file.sync_all()
        })
    }

    pub fn source_path(&self) -> &Option<PathBuf>
    {
        &self.src_path
    }

    pub fn insert(&mut self, c: char) -> Result<(), &'static str>
    {
        let (cx, cy) = self.get_cursor();
        if let Some(line) = self.content.get_mut(cy as usize) {
            line.insert(cx as usize, c);
            self.move_cursor(Relative(1, 0));
            Ok(())
        } else {
            Err("line not available")
        }
    }

    pub fn insert_newline(&mut self) -> Result<(), &'static str>
    {
        let (cx, cy) = self.get_cursor();
        let line = self.content.get_mut(cy as usize);
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
            self.content.insert((cy + 1) as usize, right);
        }

        self.move_cursor(Absolute(0, cy + 1));

        Ok(())
    }

    pub fn remove(&mut self) -> Result<(), &'static str>
    {
        let (cx, cy) = self.get_cursor();
        let (nx, ny) = (cx - 1, cy - 1);

        if let Some(line) = self.content.get_mut(cy as usize) {
            // TODO: that is disgusting
            let len = line.len() as i64;
            if 0 <= nx && nx < len {
                line.remove(nx as usize);
                self.move_cursor(Relative(-1, 0));
            }
            if nx < 0 && 0 <= ny && 1 < self.content.len() {
                let removed = self.content.remove(cy as usize);
                self.move_cursor(EndOfRow(ny));
                if len != 0 {
                    self.content
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

    pub fn get_row_at(&self, line: usize) -> Option<&str>
    {
        self.content.get(line).and_then(|c| Some(c.as_ref()))
    }

    pub fn get_cursor(&self) -> Position
    {
        self.cursor
    }

    pub fn move_cursor(&mut self, mv: CursorMove)
    {
        let (x, y) = match mv {
            Absolute(x, y) => (x, y),
            EndOfRow(y) => (0, y),
            CurrentRow(x) => (x, self.get_cursor().1),
            Relative(rx, ry) => {
                let (cx, cy) = self.get_cursor();
                (cx + rx, cy + ry)
            }
        };
        if let Some(line) = self.content.get(y as usize) {
            self.cursor.1 = y;
            let len = line.len() as i64;

            match mv {
                EndOfRow(_) => self.cursor.0 = len,
                _ => {
                    if 0 <= x {
                        self.cursor.0 = x;
                        if len < self.cursor.0 {
                            self.cursor.0 = len;
                        }
                    }
                }
            }
        }
    }
}
