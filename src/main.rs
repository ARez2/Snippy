use crossterm::{
    event::{self, *},
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
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
extern crate clipboard;
use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use snippy::{app::{App, InputMode, NewSnippetMode}, snippet::CodeSnippet};

const ORANGE: Color = Color::Rgb(252, 141, 0);
const SAVEFILE_PATH: &str = "savestate.snippy";

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // load the app state from the save file if that exists, else create new App
    let mut app = match Path::new(SAVEFILE_PATH).exists() {
        true => {
            load_app_state()
        },
        false => {
            App::default()
        },
    };
    let res = run_app(&mut terminal, &mut app);
    save_app_state(&app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        EnableBracketedPaste
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}


fn save_app_state(app: &App) {
    let app_serialized = serde_json::to_string(app).unwrap();
    let mut file = File::create(SAVEFILE_PATH).unwrap();
    file.write_all(app_serialized.as_bytes()).unwrap();
}

fn load_app_state() -> App {
    let mut file = File::open(SAVEFILE_PATH).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let app_deserialized : App = serde_json::from_str(contents.as_str()).unwrap();
    app_deserialized
}



fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: &mut App) -> io::Result<()> {
    loop {
        let mut new_input_mode = &app.input_mode;
        let mut clear_found_snippets = false;
        let mut push_current_snippet = false;
        // (list idx, snippet idx)
        let mut found_indices = Vec::<(usize, usize)>::new();
        let mut delete_snippet = None;
        
        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => {
                    match key.code {
                        KeyCode::Char('n') => {
                            new_input_mode = &InputMode::NewSnippet(NewSnippetMode::TypeName);
                            app.current_snippet = Some(CodeSnippet::new(app.return_next_idx()));
                            app.input = String::new();
                        }
                        KeyCode::Char('f') => {
                            new_input_mode = &InputMode::Search;
                            clear_found_snippets = true;
                        }
                        KeyCode::Char('c') => {
                            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                            let selected_snippet = app.found_snippets.state.selected();
                            if let Some(selected_snip_idx) = selected_snippet {
                                let snip = &app.found_snippets.items[selected_snip_idx];
                                ctx.set_contents(snip.code.clone().to_owned()).unwrap();
                            }
                        }
                        KeyCode::Char('x') => {
                            let selected_snippet = app.found_snippets.state.selected();
                            if let Some(selected_snip_idx) = selected_snippet {
                                if !app.snippets.is_empty() {
                                    let snip = &app.found_snippets.items[selected_snip_idx];
                                    delete_snippet = Some(snip.idx);
                                }
                            }
                        }
                        KeyCode::Enter => {
                            let selected_snippet = app.found_snippets.state.selected();
                            if let Some(selected_snip_idx) = selected_snippet {
                                let snip = &app.found_snippets.items[selected_snip_idx];
                                app.current_snippet = Some(snip.clone());
                                app.input = snip.tags.join(" ");
                                new_input_mode = &InputMode::NewSnippet(NewSnippetMode::TypeName);
                            }
                        }
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Up => {
                            app.found_snippets.previous();
                        }
                        KeyCode::Down => {
                            app.found_snippets.next();
                        }
                        KeyCode::Left => {
                            app.found_snippets.unselect();
                        }
                        _ => {}
                    };
                }
                InputMode::NewSnippet(new_mode) => {
                    let snip = app.current_snippet.as_mut();
                    if let Some(snip) = snip {
                        let input_field = match new_mode {
                            NewSnippetMode::TypeName => {
                                &mut snip.name
                            },
                            NewSnippetMode::TypeTags => {
                                &mut app.input
                            },
                            NewSnippetMode::TypeCode => {
                                &mut snip.code
                            },
                        };
                        let mut did_paste_something = false;
                        let paste_key = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);
                        if key == paste_key {
                            did_paste_something = true;
                            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                            let paste_content = ctx.get_contents().ok();
                            if let Some(paste_content) = paste_content {
                                input_field.push_str(paste_content.as_str());
                            };
                        };

                        if !did_paste_something {
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
                                                let tag_split: Vec<&str> = app.input.split(' ').collect();
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
                        };
                    } else {
                        new_input_mode = &InputMode::Normal;
                    }
                },
                InputMode::Search => {
                    let snippets: &mut Vec<CodeSnippet> = &mut app.snippets;
                    match key.code {
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
                                app.found_snippets.items = vec![];
                            } else {
                                found_indices = search_snippets(snippets, &app.input);
                            };
                        }
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Up => {
                            app.found_snippets.previous();
                        }
                        KeyCode::Down => {
                            app.found_snippets.next();
                        }
                        KeyCode::Left => {
                            app.found_snippets.unselect();
                        }
                        _ => (),
                    }
                },
            }
        };
        if clear_found_snippets {
            app.found_snippets.items = vec![];
        };
        app.input_mode = new_input_mode.to_owned();


        if push_current_snippet {
            // Save current snippet
            if let Some(current_snip) = app.current_snippet.clone() {
                // If currently editing an existing snippet
                if app.has_snippet_with_idx(current_snip.idx) {
                    app.remove_snippet(current_snip.idx);
                };
                app.snippets.push(current_snip);
            };
            app.current_snippet = None;
            found_indices = search_snippets(&mut app.snippets, &app.input);
            save_app_state(app);
        };

        // Call to delete a snippet
        if let Some(deletion_idx) = delete_snippet {
            app.remove_snippet(deletion_idx);
            let mut remove_idx_in_found = None;
            for (i, found) in found_indices.iter().enumerate() {
                if found.1 == deletion_idx {
                    remove_idx_in_found = Some(i);
                }
            };
            if let Some(remove_idx_in_found) = remove_idx_in_found {
                found_indices.remove(remove_idx_in_found);
            };
            save_app_state(app);
        };
        
        if app.input_mode == InputMode::Normal {
            found_indices = search_snippets(&mut app.snippets, "")
        };
        app.found_snippets.items.clear();
        for idx in found_indices.iter() {
            let snip = &mut app.snippets[idx.0];
            if !app.found_snippets.items.contains(&*snip) {
                app.found_snippets.items.push(snip.clone());
            }
        }
        
        terminal.draw(|f| ui(f, app))?;
    }
}

fn search_snippets(snippets: &'_ mut [CodeSnippet], input: &str) -> Vec<(usize, usize)> {
    let mut indices = Vec::<(usize, usize)>::new();
    let input_lower = input.to_lowercase();
    let input_lower = input_lower.as_str();
    for (snippet_idx, snippet) in snippets.iter().enumerate() {
        for tag in snippet.tags.iter() {
            let name_lower = snippet.name.to_lowercase();
            let tag_lower = tag.to_lowercase();
            if (tag_lower.contains(input_lower) || name_lower.contains(input_lower)) && !indices.contains(&(snippet_idx, snippet.idx)) {
                indices.push((snippet_idx, snippet.idx));
            };
        };
    };
    indices
}



fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(6), // Title
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

    
    let keybinds_style = Style::default();
    let snippy_title = vec![
        Spans::from(
            Span::styled("Snippy", Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED)
                .fg(ORANGE)
            )
        ),
        Spans::from(
            Span::styled("Shortcuts:", Style::default()),
        ),
        Spans::from(
            vec![Span::styled("    - ESC", keybinds_style), Span::styled(" to quit", Style::default())]
        ),
        Spans::from(
            vec![Span::styled("    - f", keybinds_style), Span::styled(" to search", Style::default())]
        ),
        Spans::from(
            vec![Span::styled("    - c", keybinds_style), Span::styled(" to copy the selected snippet", Style::default())]
        ),
        Spans::from(
            vec![Span::styled("    - x", keybinds_style), Span::styled(" to delete the selected snippet", Style::default())]
        ),
    ];
    let snippy_text = Text::from(snippy_title);
    let app_title = Paragraph::new(snippy_text);
    f.render_widget(app_title, title_chunk);

    match app.input_mode {
        InputMode::Normal | InputMode::Search => {
            let (mut title, mut t_color) = ("Normal Mode - Press f to go into Search Mode", Color::White);
            if app.input_mode == InputMode::Search {
                (title, t_color) = ("Search Mode - Press Enter to go back to Normal Mode", Color::Yellow);
            }
                
            
            // Draw Search field
            input_field(f, &String::from(title), t_color, &app.input, true, &search_chunk);
            
            let unselected_text_style = Style::default()
                .add_modifier(Modifier::UNDERLINED);
            let unselected_style = Style::default()
                .bg(Color::Rgb(32, 33, 38));
            
            let selected_style = Style::default()
                .add_modifier(Modifier::UNDERLINED)
                .add_modifier(Modifier::BOLD)
                .bg(Color::Rgb(60, 63, 71));
            
            let items: Vec<ListItem> = app
                .found_snippets
                .items
                .iter()
                .map(|snip| {
                    let lines = vec![Spans::from(Span::styled(
                        format!("{}, Tags: [{}]", snip.name, snip.tags.join(", ")),
                        unselected_text_style,
                    ))];
                    ListItem::new(lines).style(unselected_style)
                })
                .collect();

            // Create a List from all list items and highlight the currently selected one
            let items = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(Span::styled("Snippets", Style::default().add_modifier(Modifier::BOLD))))
                .highlight_style(selected_style)
                .highlight_symbol(">> ");

            // We can now render the item list
            f.render_stateful_widget(items, found_chunk, &mut app.found_snippets.state);
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

fn input_field<B: Backend>(f: &mut Frame<B>, input_title: &String, title_color: Color, input: &str, set_cursor: bool, render_area: &Rect) {
    let txt = Span::styled(input_title, Style::default()
        .fg(title_color)
        .add_modifier(Modifier::BOLD)
    );
    let mut input_para = Paragraph::new(input)
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title(Spans::from(txt)));
    
    let mut len_measure = input.len();
    let lines = input.split('\n');
    let mut line_count = lines.clone().count() as u16;
    line_count = std::cmp::max(line_count, 1);
    let offset = 3;
    let mut did_scroll = false;
    if line_count > render_area.height - 2 {
        let scoll_amt = (line_count + offset) - render_area.height;
        input_para = input_para.scroll((scoll_amt as u16, 0));
        did_scroll = true;
    }
    f.render_widget(input_para, *render_area);
    if set_cursor {
        if let Some(last_line) = lines.last() {
            len_measure = last_line.len();
            if last_line.ends_with('\n') {
                line_count += 1;
            }
        }
        let mut y_val = render_area.y + line_count as u16;
        if did_scroll {
            y_val = render_area.y + render_area.height - offset;
        }
        f.set_cursor(
            render_area.x + len_measure as u16 + 1,
            y_val,
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