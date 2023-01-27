/*
 * vSMTP mail transfer agent
 * Copyright (C) 2022 viridIT SAS
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see https://www.gnu.org/licenses/.
 *
 */
use crate::cli::args::Commands;
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    //layout::{Layout, Constraint, Direction},
    Terminal,
    //Frame,
};
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers, KeyEventKind, KeyEventState},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[allow(clippy::multiple_inherent_impl)]
impl Commands {
    /// setup crossterm terminal 
    /// # Errors
    /// possible error with terminal
    /// # Panics
    /// 
    /// TODO
    /// 
    #[inline] pub fn ui() -> Result<(), std::io::Error> {
        // crate terminal
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title("Running vqueue (/var/spool/vsmtp)")
                .borders(Borders::ALL);
            f.render_widget(block, size);
            
        })?;
        // replace by event key escape
        match read().unwrap() {
            Event::Key(KeyEvent{
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE
            }) => {
                // restore terminal
                disable_raw_mode()?;
                execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                )?;
                terminal.show_cursor()?;
            },
            Event::Key(KeyEvent{
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE
            }) => {
                // ici faire la selection entre les différentes QUEUE
            },
            Event::Key(KeyEvent{
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE
            }) => {
                // ici faire la selection entre les différentes QUEUE
            },
            _ =>{}
        };
    Ok(())
    }
}
