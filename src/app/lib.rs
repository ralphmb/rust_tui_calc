// my structs and enums
use num_parser::{self, settings};
// App is instantiated using Default, so using a wrapper around our num_parser::Context allows us to define a custom default to be instantiated
// However it's annoying to keep typing self.ctxt.0 to access the actual Context
// So we impl Deref, DerefMut. Now &self.ctxt gives a reference to the inner context etc.
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct ContextWrapper<T>(T);
impl Default for ContextWrapper<num_parser::Context> {
    fn default() -> Self {
        ContextWrapper(num_parser::Context::new(
            settings::Rounding::Round(5),
            settings::AngleUnit::Radian,
            settings::DepthLimit::Limit(100),
        ))
    }
}
impl<T> Deref for ContextWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for ContextWrapper<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum AppMode {
    #[default]
    Normal,
    Option,
    RoundingSelect,
}

// Scroll directions - used to send messages to the scroller about how it'll change state. Could be a boolean, this is maybe clearer.
pub enum ScrollDir {
    Up,
    Down,
}

pub enum CursorDir {
    Left,
    Right,
}

// Initially I tried to keep track of cursor position in a usize, and the input field was just a String.
// That was a nightmare, index hell. This seems much more stable, worked right out of the gate.
// Moving the cursor to the right means shifting the whole `after` String in memory, but it's not going to be big enough for that to ever really matter.
#[derive(Debug, Default)]
pub struct Input {
    before: String,
    after: String,
}
impl Input {
    pub fn insert(&mut self, c: char) {
        self.before.push(c);
    }
    pub fn shift(&mut self, dir: CursorDir) {
        match dir {
            CursorDir::Left => match self.before.pop() {
                Some(c) => self.after.insert(0, c),
                None => (),
            },
            CursorDir::Right => {
                if self.after.len() > 0 {
                    self.before.push(self.after.remove(0))
                }
            }
        }
    }
    pub fn get_text(&self) -> String {
        format!("{}{}", &self.before, &self.after)
    }
    pub fn get_lens(&self) -> (usize, usize) {
        // used to render the little ^ cursor under the input field
        (self.before.len(), self.after.len())
    }
    pub fn backspace(&mut self) {
        let _ = self.before.pop();
    }
    pub fn reset(&mut self) {
        self.before = String::new();
        self.after = String::new();
    }
    pub fn replace(&mut self, new_str: String) {
        // used when scrolling up into the history.
        self.after = String::new();
        self.before = new_str;
    }
}
pub enum HistoryEntry {
    Query(usize),
    Value(usize),
}
#[derive(Debug, Default)]
pub struct Queries {
    contents: Vec<(String, String)>,
    pos: usize,
    temp: Option<String>,
}
impl Queries {
    pub fn try_store(&mut self, s: String) {
        match self.temp {
            None => self.temp = Some(s),
            Some(_) => (),
        };
    }
    pub fn try_restore(&mut self) -> Option<String> {
        std::mem::take(&mut self.temp)
    }
    pub fn shift(&mut self, dir: ScrollDir) {
        let max_len = self.contents.len();
        self.pos = match dir {
            ScrollDir::Up => std::cmp::min(self.pos + 1, max_len) as usize,
            // bit messy but temporarily casting to i32 allows us to subtract 1 from 0.
            ScrollDir::Down => std::cmp::max(self.pos as i32 - 1i32, 0i32) as usize,
        };
    }

    pub fn scroll_reset(&mut self) {
        self.pos = 0;
    }
    pub fn curr(&mut self) -> Option<String> {
        // Any external calls to retrieve() can use zero-based indexing to access elements from the end of contents
        // This internal call uses zero to represent a non-scrolling state, so n-1 gives the last element of the history
        match self.pos {
            0 => None,
            n => Some(self.retrieve(HistoryEntry::Query(n - 1)).clone()),
        }
    }
    fn from_end(&self, index: usize) -> usize {
        std::cmp::max((self.contents.len() - index - 1) as i32, 0) as usize
    }
    pub fn retrieve(&self, entry: HistoryEntry) -> &String {
        match entry {
            HistoryEntry::Query(n) => &self.contents[self.from_end(n)].0,
            HistoryEntry::Value(n) => &self.contents[self.from_end(n)].1,
        }
    }
    pub fn archive(&mut self, input: String, output: String) {
        self.contents.push((input, output));
    }
    pub fn render_all(&self) -> Vec<String> {
        self.contents
            .iter()
            .rev()
            .map(|(a, b)| format!("\n {} = {}", a, b))
            .collect()
    }
    pub fn get_pos(&self) -> usize {
        self.pos
    }
}
