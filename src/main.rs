// TODO: UI
// TODO: Copy button ("quick copy")
// TODO: Delete existing Snippet
// TODO: Edit Snippet

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect, Margin},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear, BorderType},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;


#[derive(Clone, PartialEq)]
struct CodeSnippet {
    tags: Vec<String>,
    name: String,
    code: String,
}
impl CodeSnippet {
    fn new() -> CodeSnippet {
        CodeSnippet { tags: vec![],
            name: "Unnamed Code Snippet".to_string(),
            code: "".to_string(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum NewSnippetMode {
    TypeName,
    TypeTags,
    TypeCode,
}


#[derive(Clone, PartialEq)]
enum InputMode {
    Normal,
    Search,
    NewSnippet(NewSnippetMode),
}

/// App holds the state of the application
#[derive(Clone)]
struct App {
    input: String,
    input_mode: InputMode,
    snippets: Vec<CodeSnippet>,
    /// Found snippets displayed when searching
    found_snippets: Vec<CodeSnippet>,
    // Currently edited snippet
    current_snippet: Option<CodeSnippet>,
}

impl Default for App {
    fn default() -> App {
        let mut example_snippet = CodeSnippet::new();
        example_snippet.name = "Example Snippet #1".to_string();
        example_snippet.code = "enum InputMode {
            Normal,
            Search,
            NewSnippet,
        }".to_string();
        example_snippet.tags = vec!["example".to_string(), "bro".to_string()];
        App {
            input: String::new(),
            input_mode: InputMode::Search,
            snippets: vec![example_snippet],
            found_snippets: vec![],
            current_snippet: None,
        }
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::default();
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;
        let snippets: &mut Vec<CodeSnippet> = &mut app.snippets;
        
        let mut new_input_mode = &app.input_mode;
        let mut clear_found_snippets = false;
        let mut push_current_snippet = false;
        let mut found_indices = Vec::<usize>::new();
        
        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => {
                    match key.code {
                        KeyCode::Char('n') => {
                            new_input_mode = &InputMode::NewSnippet(NewSnippetMode::TypeName);
                            app.current_snippet = Some(CodeSnippet::new());
                            app.input = String::new();
                        }
                        KeyCode::Char('f') => {
                            new_input_mode = &InputMode::Search;
                            clear_found_snippets = true;
                        }
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        _ => {}
                    };
                }
                InputMode::NewSnippet(new_mode) => {
                    let snip = app.current_snippet.as_mut().unwrap();
                    let input_field;
                    match new_mode {
                        NewSnippetMode::TypeName => {
                            input_field = &mut snip.name;
                        },
                        NewSnippetMode::TypeTags => {
                            input_field = &mut app.input;
                        },
                        NewSnippetMode::TypeCode => {
                            input_field = &mut snip.code;
                        },
                    }
                    match key.code {
                        KeyCode::Esc => {
                            new_input_mode = &InputMode::Normal;
                        },
                        KeyCode::Char(c) => {
                            input_field.push(c);
                        }
                        KeyCode::Backspace => {
                            input_field.pop();
                        },
                        KeyCode::Enter => {
                            match new_mode {
                                NewSnippetMode::TypeName => {
                                    new_input_mode = &InputMode::NewSnippet(NewSnippetMode::TypeTags);
                                },
                                NewSnippetMode::TypeTags => {
                                    if let Some(current_snip) = &mut app.current_snippet {
                                        let tag_split: Vec<&str> = app.input.split(" ").collect();
                                        let mut new_tags = vec![];
                                        for t in tag_split {
                                            new_tags.push(String::from(t));
                                        };
                                        current_snip.tags = new_tags;
                                    }
                                    new_input_mode = &InputMode::NewSnippet(NewSnippetMode::TypeCode);
                                },
                                NewSnippetMode::TypeCode => {
                                    if key.modifiers == KeyModifiers::ALT {
                                        push_current_snippet = true;

                                        new_input_mode = &InputMode::Normal;
                                        app.input = String::new();
                                    } else {
                                        input_field.push('\n');
                                    }
                                },
                            };
                        }
                        KeyCode::Tab => {
                            for _i in 0..4 {
                                input_field.push(' ');
                            }
                        }
                        KeyCode::BackTab => {
                            let inp = input_field.clone();
                            let lines = inp.lines();
                            if let Some(lastline) = lines.last() {
                                if lastline.starts_with('\t') {
                                    input_field.pop();
                                } else if lastline.ends_with("    ") {
                                    for _i in 0..4 {
                                        input_field.pop();
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                },
                InputMode::Search => match key.code {
                    KeyCode::Enter => {
                        new_input_mode = &InputMode::Normal;
                        found_indices = search_snippets(snippets, &app.input);
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                        found_indices = search_snippets(snippets, &app.input);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                        if app.input.is_empty() {
                            app.found_snippets = vec![];
                        } else {
                            found_indices = search_snippets(snippets, &app.input);
                        };
                    }
                    KeyCode::Esc => {
                        return Ok(());
                    }
                    _ => (),
                },
            }
        };
        if clear_found_snippets {
            app.found_snippets = vec![];
        };
        app.input_mode = new_input_mode.to_owned();
        
        for idx in found_indices {
            let snip = &mut snippets[idx];
            if !app.found_snippets.contains(&&*snip) {
                app.found_snippets.push(snip.clone());
            }
        }
        if push_current_snippet {
            // Save current snippet
            if let Some(current_snip) = app.current_snippet.clone() {
                snippets.push(current_snip);
            };
            app.current_snippet = None;
        };
        
        
        terminal.draw(|f| ui(f, &app))?;
        
    }
}



fn search_snippets<'a>(snippets: &'a mut Vec<CodeSnippet>, input: &String) -> Vec<usize> {
    let mut indices = Vec::<usize>::new();
    for (snippet_idx, snippet) in snippets.iter().enumerate() {
        for tag in snippet.tags.iter() {
            if tag.contains(input.as_str()) && !indices.contains(&snippet_idx) {
                indices.push(snippet_idx);
            };
        };
        if snippet.name.contains(input.as_str()) && !indices.contains(&snippet_idx) {
            indices.push(snippet_idx);
        }
    };
    indices
}


fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1), // Title
                Constraint::Length(1), // Just a bit of space
                Constraint::Length(3), // Search field
                Constraint::Max(2), // Found snippets field
            ]
            .as_ref(),
        )
        .split(f.size());
    let title_chunk = chunks[0];
    let search_chunk = chunks[2];
    let found_chunk = chunks[3];


    let snippy_title = vec![
        Span::styled("Snippy", Style::default()
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED)
            .fg(Color::Rgb(252, 141, 0))
        ),
        Span::raw("  Press ESC to quit"),
    ];
    let snippy_text = Text::from(Spans::from(snippy_title));
    let app_title = Paragraph::new(snippy_text);
    f.render_widget(app_title, title_chunk);

    match app.input_mode {
        InputMode::Normal | InputMode::Search => {
            let (mut title, mut t_color) = ("Normal Mode - Press 'f' to search for snippets, 'n' to create a new Snippet", Color::White);
            match app.input_mode {
                InputMode::Search => (title, t_color) = ("Search Mode - Press Enter to go back to Normal Mode", Color::Yellow),
                _ => (),
            };
            
            // Draw Search field
            input_field(f, &String::from(title), t_color, &app.input, true, &search_chunk);
            
            // Draw found snippets list
            let snippet_list: Vec<ListItem> = app.found_snippets
                .iter()
                .enumerate()
                .map(|(_, m)| {
                    let content = vec![Spans::from(Span::raw(format!("{}", m.name)))];
                    ListItem::new(content)
                })
                .collect();
            let found_snippets =
                List::new(snippet_list).block(Block::default().borders(Borders::ALL).title("Snippets"));
            f.render_widget(found_snippets, found_chunk);
        },
        InputMode::NewSnippet(new_mode) => {
            let block = Block::default()
                .title("New Snippet. Press ESC to close this popup")
                .borders(Borders::all())
                .border_style(Style::default().fg(Color::Cyan))
                .border_type(BorderType::Double);
            let area = centered_rect(90, 90, f.size());
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(block, area);
            
            let margin_obj = Margin { vertical: 1, horizontal: 2};
            let inner_area = area.inner(&margin_obj);
            let inner_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3), // Name Input
                        Constraint::Length(3), // Tags Input
                        Constraint::Length(3), // Tags Input
                    ]
                    .as_ref(),
                )
                .split(inner_area);
            let name_chunk = inner_chunks[0];
            let tags_chunk = inner_chunks[1];
            let code_chunk = inner_chunks[2];
            
            if let Some(current_snippet) = &app.current_snippet {
                let texts = [
                    String::from("Name of the Snippet"),
                    String::from("Tags (separate by space)"),
                    String::from("Code of the Snippet (press ALT-Enter to continue)"),
                ];
                input_field(f, &texts[0], Color::DarkGray, &current_snippet.name,new_mode==NewSnippetMode::TypeName, &name_chunk);
                input_field(f, &texts[1], Color::DarkGray, &app.input, new_mode==NewSnippetMode::TypeTags, &tags_chunk);
                input_field(f, &texts[2], Color::DarkGray, &current_snippet.code, new_mode==NewSnippetMode::TypeCode, &code_chunk);
            };
        }
    }
    
}



fn input_field<B: Backend>(f: &mut Frame<B>, input_title: &String, title_color: Color, input: &String, set_cursor: bool, render_area: &Rect) {
    let txt = Span::styled(input_title, Style::default()
        .fg(title_color)
    );
    let input_para = Paragraph::new(input.as_ref())
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title(Spans::from(txt)));
    f.render_widget(input_para, *render_area);
    if set_cursor {
        let mut len_measure = input.len();
        let lines = input.split('\n');
        let mut line_count = lines.clone().count() as u16;
        line_count = std::cmp::max(line_count, 1);
        if let Some(last_line) = lines.last() {
            len_measure = last_line.len();
            if last_line.ends_with('\n') {
                line_count += 1;
            }
        }
        f.set_cursor(
            render_area.x + len_measure as u16 + 1,
            render_area.y + line_count as u16,
        );
    }
}



/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}