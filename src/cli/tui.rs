// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use std::io;
use std::sync::{Arc, Mutex};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

use crate::fuzz_runner::{FuzzerStats, get_tui_stats};

#[derive(Debug)]
pub struct TuiApp {
    fuzzer_stats: Arc<Mutex<FuzzerStats>>,
    exit: bool,
}

impl TuiApp {
    pub fn new(fuzzer_stats: Arc<Mutex<FuzzerStats>>) -> Self {
        Self {
            fuzzer_stats,
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        // Poll for events with a timeout of 100ms
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            }
        } else {
            // Auto update the stats within `render()`
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &TuiApp {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("DataFusion Fuzzer Running Status".bold());

        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        let stats = get_tui_stats(&self.fuzzer_stats);

        // Create the basic stats text
        let mut lines = vec![
            Line::from(vec![
                "Rounds: ".into(),
                format!("{}/{}", stats.rounds_completed, stats.total_rounds).yellow(),
            ]),
            Line::from(vec![
                "Queries Executed: ".into(),
                stats.queries_executed.to_string().yellow(),
            ]),
            Line::from(vec![
                "Queries Success Rate: ".into(),
                stats.success_rate.to_string().yellow(),
            ]),
            Line::from(vec![
                "Slow Queries: ".into(),
                format!(
                    "{} ({:.1}%)",
                    stats.queries_slow,
                    if stats.queries_executed > 0 {
                        (stats.queries_slow as f64 / stats.queries_executed as f64) * 100.0
                    } else {
                        0.0
                    }
                )
                .yellow(),
            ]),
            Line::from(vec![
                "Queries Per Second: ".into(),
                format!("{:.2}", stats.queries_per_second).yellow(),
            ]),
            Line::from(vec![
                "Running Time: ".into(),
                format!("{:.2}s", stats.running_time_secs).yellow(),
            ]),
            Line::from(""),
            Line::from("─".repeat(40).cyan()), // Add a separator line before query
            Line::from(vec!["Recent Query:".bold().cyan().into()]),
        ];

        // Add the query lines with proper formatting
        let query_lines: Vec<Line> = stats
            .recent_query
            .lines()
            .map(|line| Line::from(line.to_string().yellow()))
            .collect();
        lines.extend(query_lines);

        // Add a separator after the query
        lines.push(Line::from("─".repeat(40).cyan()));

        // Add exit instruction hint
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            "Press ".into(),
            "<q>".bold().yellow(),
            " to exit".into(),
        ]));

        let stats_text = Text::from(lines);

        Paragraph::new(stats_text)
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}

/// Initialize the terminal for TUI
pub fn init() -> DefaultTerminal {
    ratatui::init()
}

/// Restore the terminal after TUI is done
pub fn restore() {
    ratatui::restore();
}
