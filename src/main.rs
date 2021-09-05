use anyhow::Context;
use crossterm::event::{Event, KeyCode, KeyEvent};
use rand::prelude::*;
use std::io::{BufRead, Write};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};

#[derive(serde::Deserialize)]
struct Thing {
    name: String,
    description: String,
}

fn draw_thing<B: Backend>(selected: bool, thing: &Thing, f: &mut Frame<B>, mut size: Rect) {
    let mut block = Block::default()
        .title(format!(" {} ", thing.name))
        .title_alignment(tui::layout::Alignment::Center);
    if selected {
        block = block
            .border_style(Style::default().fg(Color::Red))
            .border_type(tui::widgets::BorderType::Thick)
            .title(Span::styled(
                format!(" {} ", thing.name),
                Style::default().add_modifier(Modifier::BOLD),
            ));
    }
    f.render_widget(block.borders(Borders::ALL), size);
    size.x += 2;
    size.y += 2;
    size.width -= 4;
    size.height -= 4;
    f.render_widget(
        Paragraph::new(thing.description.as_str())
            .alignment(tui::layout::Alignment::Center)
            .wrap(Wrap { trim: false }),
        size,
    );
}

fn info(text: String) -> anyhow::Result<()> {
    let mut terminal = Terminal::new(tui::backend::CrosstermBackend::new(std::io::stdout()))?;
    loop {
        terminal.clear()?;
        terminal.draw(|f| {
            let mut size = f.size();
            size.x += 5;
            size.y += 5;
            size.width -= 10;
            size.height -= 10;
            f.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
                size,
            );
            size.x += 2;
            size.y += 2;
            size.width -= 4;
            size.height -= 4;
            f.render_widget(
                Paragraph::new(format!("{}\n\nPress Enter", text))
                    .alignment(tui::layout::Alignment::Center),
                size,
            );
        })?;
        match crossterm::event::read()? {
            Event::Key(event) => {
                if event.code == KeyCode::Enter {
                    return Ok(());
                }
            }
            _ => {}
        }
    }
}

fn choose_one(a: Thing, b: Thing) -> anyhow::Result<Thing> {
    let mut terminal = Terminal::new(tui::backend::CrosstermBackend::new(std::io::stdout()))?;
    let mut selection = None;
    loop {
        terminal.clear()?;
        terminal.draw(|f| {
            let mut size = f.size();
            size.x += 5;
            size.y += 5;
            size.width -= 10;
            size.height -= 10;
            f.render_widget(
                Block::default()
                    .title(" What is the most painful one between these two: ")
                    .title_alignment(tui::layout::Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
                size,
            );
            size.x += 2;
            size.y += 2;
            size.width -= 4;
            size.height -= 4;
            let boxes = Layout::default()
                .constraints(
                    [
                        Constraint::Percentage(40),
                        Constraint::Percentage(20),
                        Constraint::Percentage(40),
                    ]
                    .as_ref(),
                )
                .direction(tui::layout::Direction::Horizontal)
                .split(size);

            draw_thing(selection == Some(0), &a, f, boxes[0]);
            draw_thing(selection == Some(1), &b, f, boxes[2]);
        })?;
        match crossterm::event::read()? {
            Event::Key(event) => {
                if event.code == KeyCode::Left {
                    selection = Some(0);
                }
                if event.code == KeyCode::Right {
                    selection = Some(1);
                }
                if event.code == KeyCode::Enter {
                    if let Some(selection) = selection {
                        return match selection {
                            0 => Ok(a),
                            1 => Ok(b),
                            _ => unreachable!(),
                        };
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut all_things: Vec<Thing> = Vec::new();

    for entry in std::fs::read_dir("data")? {
        let entry = entry?;
        if entry.metadata()?.is_file() {
            let path = entry.path();
            if path.extension() == Some("json".as_ref()) {
                let thing: Thing = serde_json::from_reader(std::fs::File::open(path)?)?;
                all_things.push(thing);
            }
        }
    }
    all_things.shuffle(&mut rand::thread_rng());

    let mut round = 1;
    while all_things.len() > 1 {
        info(format!(
            "Round {}. {} languages left",
            round,
            all_things.len()
        ))?;
        let mut next_round_things = Vec::new();
        let mut all_things_iter = all_things.into_iter();
        while let Some(thing) = all_things_iter.next() {
            if let Some(other_thing) = all_things_iter.next() {
                let choice = choose_one(thing, other_thing)?;
                next_round_things.push(choice);
            } else {
                next_round_things.push(thing);
            }
        }
        all_things = next_round_things;
        round += 1;
    }

    let winner = all_things.pop().unwrap();
    info(format!(
        "The winner is {}\n\nIt is the the language that brings most pain and suffering. GG",
        winner.name
    ))?;

    Ok(())
}
