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
extern crate alloc;
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, BorderType, Tabs, Paragraph, List, ListItem, ListState},
    layout::{Layout, Constraint, Direction, Alignment},
    text::{Span, Spans},
    style::{Color, Modifier, Style},
    Terminal,
};
use crossterm::{
    event,
    event::{Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[derive(Copy, Clone)]
enum MenuItem {
    Home,
    Vqueue,
}
#[derive(Debug)]
enum SelectedQueue {
    Nothing,
    Dead,
    Deferred,
    Delegated,
    Deliver,
    Working,
}

impl SelectedQueue {
    fn next(&self) -> SelectedQueue {
        match self {
            SelectedQueue::Nothing => SelectedQueue::Dead,
            SelectedQueue::Dead => SelectedQueue::Deferred,
            SelectedQueue::Deferred => SelectedQueue::Delegated,
            SelectedQueue::Delegated => SelectedQueue::Deliver,
            SelectedQueue::Deliver => SelectedQueue::Working,
            SelectedQueue::Working => SelectedQueue::Dead,
        }
    }
    fn previous(&self) -> SelectedQueue {
        match self {
            SelectedQueue::Nothing => SelectedQueue::Working,
            SelectedQueue::Dead => SelectedQueue::Working,
            SelectedQueue::Deferred => SelectedQueue::Dead,
            SelectedQueue::Delegated => SelectedQueue::Deferred,
            SelectedQueue::Deliver => SelectedQueue::Delegated,
            SelectedQueue::Working => SelectedQueue::Deliver,
        }
    }
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
#[derive(Clone)]
struct MessageList<'a> {
    message: Vec<ListItem<'a>>,
}

impl<'a> MessageList<'a> {
    fn new(message: Vec<ListItem<'a>>) -> MessageList<'a> {
        MessageList {
            message,
        }
    }
}

struct QueueList<'a> {
    state: ListState,
    queues: Vec<ListItem<'a>>,
}

impl<'a> QueueList<'a> {
    fn new(queues: Vec<ListItem>) -> QueueList {
        QueueList { 
            state: ListState::default(),
            queues,
        }
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.queues.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.queues.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
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
    #[inline] pub async fn ui(queue_manager: &alloc::sync::Arc<impl GenericQueueManager>,) -> anyhow::Result<()> {
        let mut selected_queue = SelectedQueue::Nothing;
        // crate terminal
        enable_raw_mode()?;
        let menu_titles = vec!["Home", "Vqueue","Escape"];
        let mut active_menu_item = MenuItem::Home;
        
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let dead_list = queue_manager.list(&QueueID::Dead).await?.into_iter().collect::<anyhow::Result<Vec<String>>>()?;
        let deferred_list = queue_manager.list(&QueueID::Deferred).await?.into_iter().collect::<anyhow::Result<Vec<String>>>()?;
        let delegated_list = queue_manager.list(&QueueID::Delegated).await?.into_iter().collect::<anyhow::Result<Vec<String>>>()?;
        let deliver_list = queue_manager.list(&QueueID::Deliver).await?.into_iter().collect::<anyhow::Result<Vec<String>>>()?;
        let working_list = queue_manager.list(&QueueID::Working).await?.into_iter().collect::<anyhow::Result<Vec<String>>>()?;
        
        let mut queue_list = QueueList::new(vec![
            ListItem::new("Dead"),
            ListItem::new("Deferred"),
            ListItem::new("Delegated"),
            ListItem::new("Deliver"),
            ListItem::new("Working"),
        ]);
        let dead_message_list = MessageList::new(
            dead_list.iter().map(|message| ListItem::new(message.as_str()))
            .collect()
        );
        let deferred_message_list = MessageList::new(
            deferred_list.iter().map(|message| ListItem::new(message.as_str()))
            .collect()
        );
        let delegated_message_list = MessageList::new(
            delegated_list.iter().map(|message| ListItem::new(message.as_str()))
            .collect()
        );
        let deliver_message_list = MessageList::new(
            deliver_list.iter().map(|message| ListItem::new(message.as_str()))
            .collect()
        );
        let working_message_list = MessageList::new(
            working_list.iter().map(|message| ListItem::new(message.as_str()))
            .collect()
        );
        loop {
            terminal.draw(|f| {
                let mut chunks = Layout::default()
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
                        chunks = Layout::default()
                            .margin(5)
                            .direction(Direction::Horizontal)
                            .constraints(
                                [
                                Constraint::Percentage(20),
                                Constraint::Percentage(30),
                                Constraint::Percentage(50),
                                ]
                            .as_ref(),
                        )
                            .split(f.size()); 
                        let list = List::new(queue_list.queues.clone())
                            .block(Block::default().borders(Borders::ALL).title("Vqueue"))
                            .highlight_style(
                                Style::default()
                                    .add_modifier(Modifier::BOLD),
                            )
                            .highlight_symbol(">> ");
                        // We can now render the item list
                        f.render_stateful_widget(list, chunks[0], &mut queue_list.state);
                        match selected_queue {
                            SelectedQueue::Dead => {
                                f.render_widget(Self::details_page(&dead_message_list.clone()), chunks[1]);
                                let clone_dead_list = &dead_list;
                                let test = tokio::task::block_in_place(move || {
                                tokio::runtime::Handle::current().block_on(Self::message_body(&clone_dead_list, queue_manager))
                                });
                                f.render_widget(test, chunks[2]);
                            }
                            SelectedQueue::Deferred => f.render_widget(Self::details_page(&deferred_message_list.clone()), chunks[1]),
                            SelectedQueue::Delegated => f.render_widget(Self::details_page(&delegated_message_list.clone()), chunks[1]),
                            SelectedQueue::Deliver => f.render_widget(Self::details_page(&deliver_message_list.clone()), chunks[1]),
                            SelectedQueue::Working => f.render_widget(Self::details_page(&working_message_list.clone()), chunks[1]),
                            SelectedQueue::Nothing => (),
                        };
                        
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
                    KeyCode::Tab => {
                        queue_list.unselect();
                        selected_queue = SelectedQueue::Nothing;
                    }
                    KeyCode::Up => {
                        queue_list.previous();
                        selected_queue = selected_queue.previous();
                    }
                    KeyCode::Down => {
                        queue_list.next();
                        selected_queue = selected_queue.next();
                    }
                    //KeyCode::Enter
                    //KeyCode::Right
                    _ => {}
                }
            }
        };
    Ok(())
    }
    #[must_use]
    #[inline]
    ///
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
    #[must_use]
    #[inline]
    fn details_page<'b>(message_list: &MessageList<'b>) -> List<'b> {
        let details = List::new(message_list.message.clone())
            .block(Block::default().borders(Borders::ALL).title("Details"));
        details
    }
    #[inline]
    async fn message_body<'c>(message_uid: &Vec<String>, queue_manager: &alloc::sync::Arc<impl GenericQueueManager>) -> Paragraph<'c> {
        let uid = uuid::Uuid::parse_str(&message_uid[0]).unwrap();
        let message_body = queue_manager.get_msg(&uid).await.unwrap();
        let raw_body = message_body.inner().to_string();
        let paragraph_body_message = Paragraph::new(vec![
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw(raw_body)]),
            Spans::from(vec![Span::raw("")]),
        ])
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Body Message")
                .border_type(BorderType::Plain),
        );
        paragraph_body_message
    }
}
