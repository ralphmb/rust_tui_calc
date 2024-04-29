use num_parser::{
    self,
    settings::{self, AngleUnit, Rounding},
};
mod lib;
use crate::app::lib::{
    AppMode, ContextWrapper, CursorDir, HistoryEntry, Input, Queries, ScrollDir,
};
mod func;

use crate::tui;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
use std::io;

#[derive(Debug, Default)]
pub struct App {
    // Stores current text in the input field, as well as cursor position. Overwritten when scrolling through history.
    input: Input,
    // Current text in the output field, either last result or last error.
    last_output: String,
    // Vectors of user queries, if no errors occured and where no vars/functions were defined
    history: Queries,
    // Contains a num_parser context object. Stores the user variables/functions and is used for evaluating new queries. The wrapper is used to define the default initial state of the context, located in app/lib.rs
    ctxt: ContextWrapper<num_parser::Context>,
    // Enum describing app state - whether to display input/option etc. windows
    mode: AppMode,
    // Triggered on exit
    exit: bool,
}
impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.mode {
            AppMode::Normal => match key_event.code {
                // All keybindings in the normal input mode.
                KeyCode::Enter => self.evaluate(),
                KeyCode::Backspace => {
                    self.input.backspace();
                }
                KeyCode::Tab => self.mode = AppMode::Option,
                KeyCode::Char(c) => self.input.insert(c),
                KeyCode::Esc => {
                    // esc will either clear the input text or escape scrolling and reset back to prior input
                    // so either retrieve stored input from history or reset input
                    self.input
                        .replace(self.history.try_restore().unwrap_or("".to_string()));
                    self.history.scroll_reset();
                }
                KeyCode::Up => {
                    self.history.shift(ScrollDir::Up);
                    self.history.try_store(self.input.get_text());
                    match self.history.curr() {
                        None => (),
                        Some(s) => self.input.replace(s),
                    }
                }
                KeyCode::Down => {
                    self.history.shift(ScrollDir::Down);
                    match self.history.curr() {
                        None => self
                            .input
                            .replace(self.history.try_restore().unwrap_or("".to_string())),
                        Some(s) => self.input.replace(s),
                    }
                }
                KeyCode::Left => self.input.shift(CursorDir::Left),
                KeyCode::Right => self.input.shift(CursorDir::Right),
                _ => (),
            },
            AppMode::Option => match key_event.code {
                // Option mode keybinds
                KeyCode::Char('q') => self.exit(),
                // maybe I should make a nicer way to return the context from App - TODO ?
                KeyCode::Char('d') => match self.ctxt.angle_unit {
                    // num_parser has a `turn` measure, 0->1, but I'll never use it
                    // so we just toggle between degrees and radians
                    settings::AngleUnit::Radian => self.ctxt.angle_unit = AngleUnit::Degree,
                    _ => self.ctxt.angle_unit = AngleUnit::Radian,
                },
                KeyCode::Char('r') => {
                    self.input.reset();
                    self.mode = AppMode::RoundingSelect;
                }
                KeyCode::Char('c') => self.ctxt = ContextWrapper::default(),
                KeyCode::Tab => self.mode = AppMode::Normal,
                KeyCode::Esc => self.mode = AppMode::Normal,
                _ => (),
            },
            AppMode::RoundingSelect => match key_event.code {
                KeyCode::Tab => {
                    self.input.reset();
                    self.mode = AppMode::Option
                }
                KeyCode::Esc => {
                    self.input.reset();
                    self.mode = AppMode::Option
                }
                KeyCode::Char(c) => self.input.insert(c),
                KeyCode::Enter => {
                    // if input is parseable as u8 then it becomes the new rounding accuracy
                    // else we remove the rounding
                    match self.input.get_text().parse::<u8>() {
                        Ok(i) => {
                            self.ctxt.rounding = settings::Rounding::Round(std::cmp::min(16u8, i))
                        }
                        Err(_) => self.ctxt.rounding = settings::Rounding::NoRounding,
                    };
                    self.input.reset();
                    self.mode = AppMode::Normal;
                }
                KeyCode::Backspace => {
                    self.input.backspace();
                }
                KeyCode::Left => self.input.shift(CursorDir::Left),
                KeyCode::Right => self.input.shift(CursorDir::Right),
                _ => (),
            },
        }
    }
    fn evaluate(&mut self) {
        // workhorse
        // does actual evaluation of user inputs
        // eval_with_mutable_context allows user defined variables and functions
        let out = num_parser::eval_with_mutable_context(&*self.input.get_text(), &mut self.ctxt);
        self.history.scroll_reset();
        match out {
            Ok(res) => match res {
                Some(val) => {
                    // if user query is evaluated without error:
                    self.history.archive(self.input.get_text(), val.to_string());
                    self.last_output = self.history.retrieve(HistoryEntry::Value(0)).clone(); // display it in top pane
                    self.input.reset(); // clear the current input
                }
                None => {
                    // no error and no return value occurs when user inputs a variable/function definition
                    // so just clear the in/output
                    // so just clear the in/output
                    self.input.reset();
                    self.last_output = "".to_string();
                }
            },
            Err(err) => self.last_output = err.to_string(),
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // big function sorry
        // tried to make it more modular, long way to go
        // does all the layout and rendering
        // make_para() is a helper function, saves a few lines here and there on defining ratatui Paragraphs
        // the render_foo()s handle each pane
        fn make_para(contents: Text, blck: Block, location: Rect, buf: &mut Buffer) {
            Paragraph::new(contents)
                .centered()
                .alignment(Alignment::Center)
                .block(blck)
                .render(location, buf);
        }
        fn render_normal(
            inp: String,
            lens: (usize, usize),
            scroller_pos: usize,
            loc: Rect,
            buf: &mut Buffer,
        ) {
            let input_title = Title::from(" Input ".bold());
            let mut instructions_raw = vec![
                " Options:".into(),
                "<Tab> ".bold(),
                "Scroll History:".bold(),
                "<Up>/<Down> ".bold(),
            ];
            if scroller_pos != 0 {
                let mut scroll_exit = vec!["Back to input: ".bold(), "<Esc> ".bold()];
                instructions_raw.append(&mut scroll_exit);
            } else {
                let mut clear_inp = vec!["Clear input:".bold(), "<Esc> ".bold()];
                instructions_raw.append(&mut clear_inp);
            }
            let instructions = Title::from(Line::from(instructions_raw));
            let input_block = Block::default()
                .title(input_title.alignment(Alignment::Center))
                .title(
                    instructions
                        .alignment(Alignment::Center)
                        .position(Position::Bottom),
                )
                .borders(Borders::ALL)
                .border_set(border::THICK);
            make_para(
                Text::from(format!(
                    "\n:> {}\n{}^{}",
                    &*inp,
                    " ".repeat(lens.0 + 2),
                    " ".repeat(lens.1)
                )),
                input_block,
                loc,
                buf,
            );
        }
        fn render_options(loc: Rect, buf: &mut Buffer) {
            let options_title = Title::from(" Options ".bold());
            let options_instructions = Title::from(Line::from(vec![
                " Back to input mode: ".into(),
                "<Tab> ".bold(),
            ]));
            let options_block = Block::default()
                .title(options_title.alignment(Alignment::Center))
                .title(
                    options_instructions
                        .alignment(Alignment::Center)
                        .position(Position::Bottom),
                )
                .borders(Borders::ALL)
                .border_set(border::THICK);
            let options_content = Text::from(
                "\nToggle angle measure: <d>\nChange rounding precision <r>\nClear user variables/functions <c>\n\nQuit: <q>\n",
            );
            make_para(options_content, options_block, loc, buf)
        }
        fn render_rounding(inp: String, loc: Rect, buf: &mut Buffer) {
            let rounding_title = Title::from(" Options: Rounding ".bold());
            let rounding_instructions = Title::from(Line::from(vec![
                " Back to Options: ".into(),
                "<Tab> ".bold(),
            ]));
            let rounding_block = Block::default()
                .title(rounding_title.alignment(Alignment::Center))
                .title(
                    rounding_instructions
                        .alignment(Alignment::Center)
                        .position(Position::Bottom),
                )
                .borders(Borders::ALL)
                .border_set(border::THICK);
            let rounding_content =
                Text::from(format!("{}\n:> {}", "\nEnter an integer between 0 and 16 and hit <Enter> \nAny non-integer input will lead to no rounding\n", &*inp));
            make_para(rounding_content, rounding_block, loc, buf)
        }
        fn render_vars(context: &num_parser::Context, loc: Rect, buf: &mut Buffer) {
            let var_strings: Vec<String> = func::vars_to_strings(&context);
            let vars_title = Title::from(" User Variables ".bold());
            let vars_block = Block::default()
                .title(vars_title.alignment(Alignment::Center))
                .borders(Borders::ALL)
                .border_set(border::THICK);
            make_para(Text::from(&*var_strings.concat()), vars_block, loc, buf);
        }
        fn render_funcs(context: &num_parser::Context, loc: Rect, buf: &mut Buffer) {
            let func_strings: Vec<String> = func::funcs_to_strings(context);

            let funcs_title = Title::from(" User Functions ".bold());
            let funcs_block = Block::default()
                .title(funcs_title.alignment(Alignment::Center))
                .borders(Borders::ALL)
                .border_set(border::THICK);
            make_para(Text::from(&*func_strings.concat()), funcs_block, loc, buf);
        }
        fn render_output(
            last_out: &String,
            debug_text: String,
            context: &num_parser::Context,
            loc: Rect,
            buf: &mut Buffer,
        ) {
            let result_title = Title::from(" Output ".bold());
            let round_fmt = match context.rounding {
                Rounding::Round(n) => format!("{} d.p.", n.to_string()),
                Rounding::NoRounding => "None".to_string(),
            };
            let ang_fmt = match context.angle_unit {
                AngleUnit::Radian => "Rad ".to_string(),
                AngleUnit::Degree => "Deg ".to_string(),
                AngleUnit::Turn => "Turn ".to_string(),
            };
            let ctxt_settings = Title::from(Line::from(vec![
                format!(" Rounding: {}, ", round_fmt).into(),
                format!("Angle units: {}", ang_fmt).into(),
            ]));

            let result_block = Block::default()
                .title(result_title.alignment(Alignment::Center))
                .title(
                    ctxt_settings
                        .alignment(Alignment::Left)
                        .position(Position::Bottom),
                )
                .borders(Borders::ALL)
                .border_set(border::THICK);
            make_para(
                Text::from(format!("\n{}\n{}", last_out, debug_text)),
                result_block,
                loc,
                buf,
            );
        }
        fn render_history(hist_strings: Vec<String>, loc: Rect, buf: &mut Buffer) {
            let hist_title = Title::from(" History ".bold());
            let hist_block = Block::default()
                .title(hist_title.alignment(Alignment::Center))
                .borders(Borders::ALL)
                .border_set(border::THICK);
            make_para(Text::from(&*hist_strings.concat()), hist_block, loc, buf);
        }
        // LAYOUT
        let thirds = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(area);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(thirds[0]);

        let middle = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(thirds[1]);

        render_vars(&self.ctxt, left[0], buf);
        render_funcs(&self.ctxt, left[1], buf);
        render_output(
            &self.last_output,
            self.history.get_pos().to_string(),
            &self.ctxt,
            middle[0],
            buf,
        );
        render_history(self.history.render_all(), thirds[2], buf);
        match self.mode {
            AppMode::Option => render_options(middle[1], buf),
            AppMode::Normal => render_normal(
                self.input.get_text(),
                self.input.get_lens(),
                self.history.get_pos(),
                middle[1],
                buf,
            ),
            AppMode::RoundingSelect => render_rounding(self.input.get_text(), middle[1], buf),
        }
    }
}
