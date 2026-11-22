use crate::store::Store;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::sync::Arc;
use std::time::Duration;

pub fn run(store: Arc<Store>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut selected_tab = 0;

    let res = (|| -> anyhow::Result<()> {
        loop {
            let repos = store.list_repos().unwrap_or_default();
            let events = store.recent_events(15).unwrap_or_default();

            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(f.size());

                let header_text = format!(
                    " [1] Repos ({})  |  [2] Eventos ({})  |  [3] Grafo  |  'q' para sair",
                    repos.len(),
                    events.len()
                );
                let header = Paragraph::new(header_text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Colmeia TUI — Frota Git ")
                        .style(Style::default().fg(Color::Yellow)),
                );
                f.render_widget(header, chunks[0]);

                if selected_tab == 0 {
                    let items: Vec<ListItem> = repos
                        .iter()
                        .map(|(id, path, mode, sha)| {
                            let short_sha = if sha.len() >= 7 { &sha[..7] } else { sha };
                            ListItem::new(format!(
                                "#{:<3} [{:<7}] {} (HEAD: {})",
                                id, mode, path, short_sha
                            ))
                        })
                        .collect();

                    let repo_list = List::new(items).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Repositórios Registrados "),
                    );
                    f.render_widget(repo_list, chunks[1]);
                } else if selected_tab == 1 {
                    let items: Vec<ListItem> = events
                        .iter()
                        .map(|(id, ts, repo, kind, payload)| {
                            ListItem::new(format!(
                                "#{:<4} [{}] repo #{} - {}: {}",
                                id, ts, repo, kind, payload
                            ))
                        })
                        .collect();

                    let event_list = List::new(items).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Log de Eventos "),
                    );
                    f.render_widget(event_list, chunks[1]);
                } else {
                    // F46: grafo de co-mudança em braille
                    let graph = crate::graph::view(&store).unwrap_or(crate::graph::GraphView {
                        nodes: Vec::new(),
                        edges: Vec::new(),
                    });
                    let area = chunks[1];
                    let canvas_w = (area.width as usize).saturating_sub(2).clamp(10, 200);
                    let canvas_h = (area.height as usize).saturating_sub(2).clamp(4, 60);
                    let art = crate::braille::render_braille(
                        &graph.nodes,
                        &graph
                            .edges
                            .iter()
                            .map(|(a, b, w, _)| (*a, *b, *w))
                            .collect::<Vec<_>>(),
                        canvas_w,
                        canvas_h,
                    );
                    let canvas = Paragraph::new(art).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!(" Grafo ({} arestas) ", graph.edges.len())),
                    );
                    f.render_widget(canvas, chunks[1]);
                }
            })?;

            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('1') => selected_tab = 0,
                        KeyCode::Char('2') => selected_tab = 1,
                        KeyCode::Char('3') => selected_tab = 2,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    })();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}
