#![allow(clippy::enum_glob_use, clippy::wildcard_imports)]
use std::{error::Error, io, io::stdout};

use color_eyre::config::HookBuilder;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};

const TODO_HEADER_BG: Color = tailwind::BLUE.c950;
const NORMAL_ROW_COLOR: Color = tailwind::SLATE.c950;
const ALT_ROW_COLOR: Color = tailwind::SLATE.c900;
const SELECTED_STYLE_FG: Color = tailwind::BLUE.c300;
const TEXT_COLOR: Color = tailwind::SLATE.c200;


#[allow(dead_code)]
struct SSHGroup<'a> {
    path: &'a str,
    hosts: &'a str,
}
struct SSHStore<'a> {
    state: ListState,
    items: Vec<SSHGroup<'a>>,
    last_selected: Option<usize>,
}
struct App<'a> {
    stores: SSHStore<'a>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    init_error_hooks()?;
    let terminal = init_terminal()?;

    // create app and run it
    App::new().run(terminal)?;

    restore_terminal()?;

    Ok(())
}

fn init_error_hooks() -> color_eyre::Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = restore_terminal();
        error(e)
    }))?;
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic(info);
    }));
    Ok(())
}

fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> color_eyre::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

impl<'a> SSHStore<'a> {
    fn with_groups(groups: [(&'a str, &'a str); 6]) -> SSHStore<'a> {
        SSHStore {
            state: ListState::default(),
            items: groups.iter().map(SSHGroup::from).collect(),
            last_selected: None
        }
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        let offset = self.state.offset();
        self.last_selected = self.state.selected();
        self.state.select(None);
        *self.state.offset_mut() = offset;
    }
}
impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            stores: SSHStore::with_groups([
                ("app", "123"),
                ("app", "123"),
                ("app", "123"),
                ("app", "123"),
                ("app", "123"),
                ("app", "123"),

            ]),
        }
    }

    /// Changes the status of the selected list item
    fn change_status(&mut self) {
        // if let Some(i) = self.stores.state.selected() {
        //     self.stores.items[i].status = match self.items.items[i].status {
        //         Status::Completed => Status::Todo,
        //         Status::Todo => Status::Completed,
        //     }
        // }
    }

    fn go_top(&mut self) {
        self.stores.state.select(Some(0));
    }

    fn go_bottom(&mut self) {
        self.stores.state.select(Some(self.stores.items.len() - 1));
    }
}
impl<'a> From<&(&'a str, &'a str)> for SSHGroup<'a> {
    fn from((path, hosts): &(&'a str, &'a str)) -> Self {
        Self {
            path,
            hosts
        }
    }
}
impl App<'_> {
    fn render_todo(&mut self, area: Rect, buf: &mut Buffer) {
        // We create two blocks, one is for the header (outer) and the other is for list (inner).
        let outer_block = Block::default()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(TODO_HEADER_BG)
            .title("Hosts")
            .title_alignment(Alignment::Center);
        let inner_block = Block::default()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(NORMAL_ROW_COLOR);

        // We get the inner area from outer_block. We'll use this area later to render the table.
        let outer_area = area;
        let inner_area = outer_block.inner(outer_area);

        // We can render the header in outer_area.
        outer_block.render(outer_area, buf);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .stores
            .items
            .iter()
            .enumerate()
            .map(|(i, todo_item)| todo_item.to_list_item(i))
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(inner_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(SELECTED_STYLE_FG),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We can now render the item list
        // (look careful we are using StatefulWidget's render.)
        // ratatui::widgets::StatefulWidget::render as stateful_render
        StatefulWidget::render(items, inner_area, buf, &mut self.stores.state);
    }

    fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        loop {
            self.draw(&mut terminal)?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Char('q') | Esc => return Ok(()),
                        Char('h') | Left => self.stores.unselect(),
                        Char('j') | Down => self.stores.next(),
                        Char('k') | Up => self.stores.previous(),
                        Char('l') | Right | Enter => self.change_status(),
                        Char('g') => self.go_top(),
                        Char('G') => self.go_bottom(),
                        _ => {}
                    }
                }
            }
        }
    }
    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }
}
impl Widget for &mut App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create a space for header, todo list and the footer.
        let vertical = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ]);
        let [_, rest_area, _] = vertical.areas(area);

        // Create two chunks with equal vertical screen space. One for the list and the other for
        // the info block.
        let vertical = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [upper_item_list_area, _lower_item_list_area] = vertical.areas(rest_area);

        self.render_todo(upper_item_list_area, buf);
    }
}

impl SSHGroup <'_> {
        fn to_list_item(&self, index: usize) -> ListItem {
            let bg_color = match index % 2 {
                0 => NORMAL_ROW_COLOR,
                _ => ALT_ROW_COLOR,
            };
            let line = Line::styled(format!(" + {}", self.path), TEXT_COLOR);

            ListItem::new(line).bg(bg_color)
        }
}