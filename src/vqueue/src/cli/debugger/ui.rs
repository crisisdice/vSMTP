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
    widgets::{Block, Borders, BorderType, Tabs, Paragraph},
    layout::{Layout, Constraint, Direction, Alignment},
    text::{Span, Spans},
    style::{Color, Modifier, Style},
    Terminal,
};
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers, KeyEventKind, KeyEventState},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Vqueue,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Vqueue => 1,
        }
    }
}

#[allow(clippy::multiple_inherent_impl)]
impl Commands {
    /// setup crossterm terminal 
    /// # Errors
    /// possible error with terminal
    /// # Panics
    /// 
    /// TODO
    /// Ajouter x vqueue en zone de texte et les faire clickable pour afficher leurs mails
    #[inline] pub fn ui() -> Result<(), std::io::Error> {
        // crate terminal
        enable_raw_mode()?;
        let menu_titles = vec!["Home", "Vqueue","Escape"];
        let mut active_menu_item = MenuItem::Home;
        
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        terminal.draw(|f| {
        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(2),
                Constraint::Length(3),
            ]
            .as_ref(),
            )
            .split(size);
        let menu = menu_titles
            .iter()
            .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(
                    first,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::styled(rest, Style::default().fg(Color::White)),
            ])
            })
            .collect();

        let tabs = Tabs::new(menu)
            .select(active_menu_item.into())
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .divider(Span::raw("|"));

        f.render_widget(tabs, chunks[0]);

        match active_menu_item {
            MenuItem::Home => {
                f.render_widget(Self::home_page(), size)
            }
            MenuItem::Vqueue => {
                let queue_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
            }
        }
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
                code: KeyCode::Char('v'),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE
            }) => {
                active_menu_item = MenuItem::Vqueue;
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
            Event::Key(KeyEvent{
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE
            }) => {
                // ici entré dans une VQUEUE et voir les différents mail qui la composent
            }
            _ =>{}
        };
    Ok(())
    }

    pub fn home_page() -> Paragraph<'static>{
        let home = Paragraph::new(vec![
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw("Welcome")]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw("to")]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::styled(
                "UI command",
                Style::default().fg(Color::LightBlue),
            )]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw("Press 'V' to access Vqueue, 'Esc' to quit this terminal.")]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Home")
                .border_type(BorderType::Plain),
        );
        home
    }
}
