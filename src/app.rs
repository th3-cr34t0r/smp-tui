use std::io;

use crate::tui;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, symbols::border, widgets::*};

#[derive(Debug, Default)]
pub struct App {
    pub counter: u8,
    pub progress1: u16,
    pub exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        let main_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(1),
            ],
        )
        .split(frame.size());

        frame.render_widget(
            Block::new()
                .borders(Borders::TOP)
                .title(" Sigmanauts Mining Pool ")
                .title_alignment(Alignment::Center)
                .title_style(Style::default().fg(Color::LightYellow)),
            main_layout[0],
        );

        frame.render_widget(
            Block::new()
                .borders(Borders::TOP)
                .title(" v0.0.1 ")
                .title_alignment(Alignment::Right)
                .title_style(Color::Green),
            main_layout[2],
        );

        let inner_layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(main_layout[1]);

        let stats_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ],
        )
        .split(inner_layout[0]);

        frame.render_widget(Block::bordered().title(" Network Stats "), stats_layout[0]);
        frame.render_widget(Block::bordered().title(" Pool Stats "), stats_layout[1]);
        frame.render_widget(Block::bordered().title(" Miner Stats "), stats_layout[2]);

        let chart_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ],
        )
        .split(inner_layout[1]);

        // Create the datasets to fill the chart with
        let datasets = vec![
            // Line chart
            Dataset::default()
                .name("Th/s")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().magenta())
                .data(&[
                    (0.0, 9.3),
                    (1.0, 9.0),
                    (2.0, 10.0),
                    (3.0, 10.5),
                    (4.0, 12.0),
                    (5.0, 11.3),
                ]),
        ];

        // Create the X axis and define its properties
        let x_axis = Axis::default()
            .title("X Axis".red())
            .style(Style::default().white())
            .bounds([0.0, 5.0])
            .labels(vec!["0.0".into(), "2.5".into(), "5.0".into()]);

        // Create the Y axis and define its properties
        let y_axis = Axis::default()
            .title("Y Axis".red())
            .style(Style::default().white())
            .bounds([5.0, 15.0])
            .labels(vec!["5.0".into(), "10.0".into(), "15.0".into()]);

        // Create the chart and link all the parts together
        let chart = Chart::new(datasets)
            .block(Block::new().title(" Network Hashrate "))
            .x_axis(x_axis)
            .y_axis(y_axis);

        frame.render_widget(chart, chart_layout[0]);

        // frame.render_widget(
        //     Block::bordered().title(" Network Hashrate "),
        //     chart_layout[0],
        // );
        frame.render_widget(Block::bordered().title(" Pool Hashrate "), chart_layout[1]);
        frame.render_widget(Block::bordered().title(" Miner Hashrate "), chart_layout[2]);
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }
}

// impl Widget for &App {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let title = Title::from(" Counter App Tutorial ".bold());
//         let instructions = Title::from(Line::from(vec![
//             " Decrement ".into(),
//             "<Left>".blue().bold(),
//             " Increment ".into(),
//             "<Right>".blue().bold(),
//             " Quit ".into(),
//             "<Q> ".blue().bold(),
//         ]));
//         let block = Block::default()
//             .title(title.alignment(Alignment::Center))
//             .title(
//                 instructions
//                     .alignment(Alignment::Center)
//                     .position(Position::Bottom),
//             )
//             .borders(Borders::ALL)
//             .border_set(border::THICK);

//         let counter_text = Text::from(vec![Line::from(vec![
//             "Value: ".into(),
//             self.counter.to_string().yellow(),
//         ])]);

//         Paragraph::new(counter_text)
//             .centered()
//             .block(block)
//             .render(area, buf);
//     }
// }
