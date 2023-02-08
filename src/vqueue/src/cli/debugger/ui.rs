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
use crate::{GenericQueueManager, QueueID};
use crossterm::event;
extern crate alloc;

use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, BorderType, Tabs, Paragraph, List, ListItem},
    layout::{Layout, Constraint, Direction, Alignment},
    text::{Span, Spans},
    style::{Color, Modifier, Style},
    Terminal,
};
use crossterm::{
    event::{Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[derive(Copy, Clone)]
enum MenuItem {
    Home,
    Vqueue,
}

impl From<MenuItem> for usize {
    #[inline]
    fn from(input: MenuItem) -> Self {
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
    #[inline] pub fn ui(queue_manager: &alloc::sync::Arc<impl GenericQueueManager>,) -> Result<(), std::io::Error> {
        // crate terminal
        enable_raw_mode()?;
        let menu_titles = vec!["Home", "Vqueue","Escape"];
        let mut active_menu_item = MenuItem::Home;
        
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        loop {
            let queue: [ListItem<'_>; 5] = [ListItem::new("Dead"), ListItem::new("Deffered"), ListItem::new("Delegated"), ListItem::new("Deliver"), ListItem::new("Working")];
            let dead_list = queue_manager.list(&QueueID::Dead);
            let deffered_list = queue_manager.list(&QueueID::Deferred);
            let delegated_list = queue_manager.list(&QueueID::Delegated);
            let deliver_list = queue_manager.list(&QueueID::Deliver);
            let working_list = queue_manager.list(&QueueID::Working);
            terminal.draw(|f| {
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
                    .split(f.size());
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
                        f.render_widget(Self::home_page(), chunks[1]);
                    }
                    MenuItem::Vqueue => {
                        let queue_list = List::new(queue)
                            .block(Block::default().borders(Borders::ALL).title("Vqueue"))
                            .highlight_style(
                                Style::default()
                                    .bg(Color::LightGreen)
                                    .add_modifier(Modifier::BOLD),
                            )
                            .highlight_symbol(">> ");
                        // We can now render the item list
                        f.render_widget(queue_list, chunks[1]);
                    }
                };
            })?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => {
                        // restore terminal
                        disable_raw_mode()?;
                        execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        )?;
                        terminal.show_cursor()?;
                    }
                    KeyCode::Char('v') => active_menu_item = MenuItem::Vqueue,
                    KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                    _ =>{}
                }
            }
        };
    Ok(())
    }
    /// setup home page 
    /// # Errors
    /// 
    /// # Panics
    /// 
    /// TODO
    ///
    #[must_use]
    #[inline]
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
