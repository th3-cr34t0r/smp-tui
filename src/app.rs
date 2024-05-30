use std::{io, thread::sleep, time::Duration};

use crate::{database::get_network_hashrate, tui};
use crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, symbols::border, widgets::*};

#[derive(Debug, Default)]
pub struct App {
    pub counter: u8,
    pub network_hashrate: Vec<(f64, f64)>,
    pub pool_hashrate: Vec<(f64, f64)>,
    pub miner_hashrate: Vec<(f64, f64)>,
    pub first_run: bool,
    pub exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            if poll(Duration::from_millis(1000))? {
                self.handle_events()?;
            }

            terminal.draw(|frame| self.render_frame(frame))?;
            self.update_chart_data();
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
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
        frame.render_widget(
            Block::bordered().title(self.counter.to_string().yellow()),
            stats_layout[1],
        );
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

        frame.render_widget(
            self.render_chart(
                "Network Hashrate",
                Style::default().cyan(),
                &self.network_hashrate.clone(),
            ),
            chart_layout[0],
        );

        frame.render_widget(
            self.render_chart(
                "Pool Hashrate",
                Style::default().magenta(),
                &self.pool_hashrate.clone(),
            ),
            chart_layout[1],
        );
        frame.render_widget(
            self.render_chart(
                "Miner Hashrate",
                Style::default().light_green(),
                &self.miner_hashrate.clone(),
            ),
            chart_layout[2],
        );
    }

    fn render_chart<'a>(
        &'a self,
        name: &'static str,
        style: Style,
        data: &'a Vec<(f64, f64)>,
    ) -> Chart {
        // Create the datasets to fill the chart with
        let datasets = vec![
            // Line chart
            Dataset::default()
                .name(name)
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(style)
                .data(data),
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
            .bounds([5.0, 25.0])
            .labels(vec![
                "5.0".into(),
                "10.0".into(),
                "15.0".into(),
                "20.0".into(),
            ]);

        // Create the chart and link all the parts together
        Chart::new(datasets)
            .block(Block::new())
            .x_axis(x_axis)
            .y_axis(y_axis)
    }

    fn update_chart_data(&mut self) {
        self.network_hashrate = get_network_hashrate();
        self.pool_hashrate = get_network_hashrate();
        self.miner_hashrate = get_network_hashrate();
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
