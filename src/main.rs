use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Scrollbar, ScrollbarOrientation};
use ratatui::{Frame, Terminal};

use tui_tree_widget::{Tree, TreeItem, TreeState};
use uuid::{Uuid};

struct App<'a> {
    state: TreeState<String>,
    items: Vec<TreeItem<'a, String>>,
}
struct SSHGroup<'a> {
    path: &'a str,
    hosts: Vec<&'a str>,
    groups: Vec<SSHGroup<'a>>
}

fn to_tree_item(group: SSHGroup) -> Vec<TreeItem<String>> {
    let mut folder_or_hosts = Vec::new();
    let mut folders = Vec::new();
    for child_group in group.groups {
        if child_group.path == "/" {
            folders.extend(folder_or_hosts.clone())
        } else {
            folder_or_hosts.extend(to_tree_item(child_group))
        }
    }
    for host in &group.hosts {
        let identifier = Uuid::new_v4().to_string();
        let new_leaf = TreeItem::new_leaf(identifier, *host);
        folder_or_hosts.push(new_leaf)
    }
    if group.path == "/" {
        return folder_or_hosts
    } else {
        let identifier = Uuid::new_v4().to_string();
        let root = TreeItem::new(identifier, group.path, folder_or_hosts);
        folders.push(root.unwrap());
        return folders
    }
}
impl<'a> App<'a> {
    fn new() -> Self {
        let group = SSHGroup{
                path: "/",
                hosts: vec!["app.com", "hello.com"],
                groups: vec![ SSHGroup{
                    path: "hello/",
                    hosts: vec!["test.com", "te.com"],
                    groups: vec![ SSHGroup{
                        path: "hello/",
                        hosts: vec!["test.com", "te.com"],
                        groups: vec![],
                    }],
                },
              SSHGroup{
                  path: "app/",
                  hosts: vec!["test.com", "te.com"],
                  groups: vec![],
              }]
            };
        let  items = to_tree_item(group);
        Self {
            state: TreeState::default(),
            items
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.size();
        let widget = Tree::new(self.items.clone())
            .expect("all item identifiers are unique")
            .block(
                Block::bordered()
                    .title("SSH Client")
                    .title_bottom(format!("{:?}", self.state)),
            )
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            ))
            .highlight_style(
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        frame.render_stateful_widget(widget, area, &mut self.state);
    }
}

fn main() -> std::io::Result<()> {
    // Terminal initialization
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    // App
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    terminal.draw(|frame| app.draw(frame))?;
    loop {
        let update = match crossterm::event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Enter => app.state.toggle_selected(),
                KeyCode::Left => app.state.key_left(),
                KeyCode::Right => app.state.key_right(),
                KeyCode::Down => app.state.key_down(&app.items),
                KeyCode::Up => app.state.key_up(&app.items),
                KeyCode::Esc => app.state.select(Vec::new()),
                KeyCode::Home => app.state.select_first(&app.items),
                KeyCode::End => app.state.select_last(&app.items),
                KeyCode::PageDown => app.state.scroll_down(3),
                KeyCode::PageUp => app.state.scroll_up(3),
                _ => false,
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => app.state.scroll_down(1),
                MouseEventKind::ScrollUp => app.state.scroll_up(1),
                _ => false,
            },
            _ => false,
        };
        if update {
            terminal.draw(|frame| app.draw(frame))?;
        }
    }
}