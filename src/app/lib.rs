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

#[derive(Debug, Default)]
pub struct Scroller {
    position: usize,
    max_len: usize,
    temp: Option<String>,
}
impl Scroller {
    // This struct tracks scroll position.
    // After giving a direction to update(), it'll try to in-/decrease the scroll position
    // subject to 0 <= position <= max_len
    // where max_len is the number of user queries that evaluated without error:
    // e.g. the length of the history
    pub fn update(&mut self, dir: ScrollDir) -> usize {
        self.position = match dir {
            ScrollDir::Up => std::cmp::min(self.position + 1, self.max_len) as usize,
            // bit messy but temporarily casting to i32 allows us to subtract 1 from 0.
            ScrollDir::Down => std::cmp::max(self.position as i32 - 1i32, 0i32) as usize,
        };
        self.position
    }
    pub fn inc_max(&mut self) {
        // called in the evaluate function in app.rs
        self.max_len += 1;
    }
    pub fn store(&mut self, to_store: String) {
        // while scrolling any text previously in the input is stored in Scroller.
        // the `Some =>` arm should never trigger, but at the moment it would just silently be ignored
        // this function should only be called when a value actually needs to be stored
        // TODO - check this works when esc-ing out of a scroll - is there any way it can restore inputs that it shouldn't?
        match self.temp {
            None => self.temp = Some(to_store),
            Some(_) => (),
        }
    }
    pub fn retrieve(&mut self) -> Option<String> {
        std::mem::take(&mut self.temp)
    }
    pub fn reset(&mut self) {
        self.position = 0;
        self.temp = None;
    }
    pub fn get_pos(&self) -> usize {
        self.position
    }
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
