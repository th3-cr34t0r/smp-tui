use crate::{data::*, tui};
use crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, widgets::*};
use std::{io, time::Duration, vec};

#[derive(Debug, Default)]
pub struct App {
    address: String,
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        let mut stats = Stats::default();

        while !self.exit {
            match stats.get_data() {
                Ok(_) => self.exit = false,
                Err(_) => {
                    println!("API UNREACHABLE!");
                }
            }

            terminal.draw(|frame| self.render_frame(frame, &stats))?;

            if poll(Duration::from_secs(60))? {
                self.handle_events()?;
            }
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame, stats: &Stats) {
        let main_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
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
                .title_style(Style::default().fg(Color::Green))
                .border_style(Style::fg(Style::default().fg(Color::Green), Color::Green)),
            main_layout[0],
        );

        frame.render_widget(
            Block::new()
                .borders(Borders::TOP)
                .title(" v0.0.1 ")
                .title_alignment(Alignment::Right)
                .title_style(Color::Green)
                .border_style(Style::fg(Style::default().fg(Color::Green), Color::Green)),
            main_layout[2],
        );

        let stats_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ],
        )
        .split(main_layout[1]);

        frame.render_widget(
            Block::bordered()
                .title(" Network Stats ")
                .border_style(Style::fg(Style::default().fg(Color::Green), Color::Green)),
            stats_layout[0],
        );
        frame.render_widget(
            Block::bordered()
                .title(" Pool Stats ".green())
                .border_style(Style::fg(Style::default().fg(Color::Green), Color::Green)),
            stats_layout[1],
        );
        frame.render_widget(
            Block::bordered()
                .title(" Miner Stats ")
                .border_style(Style::fg(Style::default().fg(Color::Green), Color::Green)),
            stats_layout[2],
        );

        self.render(
            frame,
            stats_layout[0],
            vec![
                " Network Hashrate ",
                " Network Difficulty ",
                " Block Height ",
            ],
            vec![" Block Reward ", " Reward Reduction in ", " ERG Price "],
            vec![
                (stats
                    .network
                    .hashrate
                    .back()
                    .unwrap_or(&(0.0, 0.0))
                    .1
                    .to_string()
                    + " Th/s")
                    .as_str(),
                (stats.network.difficulty.to_string() + " P").as_str(),
                stats.network.height.to_string().as_str(),
            ],
            vec![
                (stats.network.reward.to_string() + " Σ").as_str(),
                stats.network.reward_reduction.to_string().as_str(),
                (stats.network.price.to_string() + " SigUSD").as_str(),
            ],
            "Network Hashrate",
            "Block",
            "Th/s",
            stats.network.hashrate.clone().into_iter().collect(),
        );

        self.render(
            frame,
            stats_layout[1],
            vec![" Pool Hashrate ", " Connected Miners ", " Current Effort "],
            vec![
                " Block found every ",
                " Blocks found ",
                " Confirming block ",
            ],
            vec![
                (stats
                    .pool
                    .hashrate
                    .back()
                    .unwrap_or(&(0.0, 0.0))
                    .1
                    .to_string()
                    + " Gh/s")
                    .as_str(),
                stats.pool.connected_miners.to_string().as_str(),
                (stats.pool.effort.to_string() + " %").as_str(),
            ],
            vec!["", stats.pool.total_blocks.to_string().as_str(), ""],
            "Pool Hashrate",
            "Block",
            "Gh/s",
            stats.pool.hashrate.clone().into_iter().collect(),
        );

        // Adding Progress Bar
        let layout_1 = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .margin(1)
        .split(stats_layout[1]);

        // Split the area in 2 segments:
        let stats_layout_progress = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .margin(1)
        .split(layout_1[0]);

        let stats_right = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ],
        )
        .split(stats_layout_progress[1]);

        frame.render_widget(
            Gauge::default()
                .block(Block::new().padding(Padding::proportional(1)))
                .gauge_style(Style::default().fg(Color::LightGreen))
                .percent(stats.pool.confirming_new_block as u16),
            stats_right[2],
        );

        self.render(
            frame,
            stats_layout[2],
            vec![
                " Current Hashrate ",
                " Average 24h Hashrate ",
                " Round Contribution ",
            ],
            vec![" Pending Shares ", " Pending Balance ", " Total Paid "],
            vec![
                (stats
                    .miner
                    .hashrate
                    .back()
                    .unwrap_or(&(0.0, 0.0))
                    .1
                    .to_string()
                    + " Mh/s")
                    .as_str(),
                stats.miner.average_hashrate.to_string().as_str(),
                stats.miner.round_contribution.to_string().as_str(),
            ],
            vec![
                stats.miner.pending_shares.to_string().as_str(),
                stats.miner.pending_balance.to_string().as_str(),
                stats.miner.total_paid.to_string().as_str(),
            ],
            "Miner Hashrate",
            "Time",
            "Mh/s",
            stats.miner.hashrate.clone().into_iter().collect(),
        );
    }

    fn render_stats(
        &self,
        frame: &mut Frame,
        area: std::rc::Rc<[Rect]>,
        title: Vec<&str>,
        value: Vec<&str>,
    ) {
        for i in 0..area.len() {
            let block = Block::bordered()
                .title(title[i])
                .title_alignment(Alignment::Center)
                .style(Style::default().green());

            let paragraph = Paragraph::new(value[i])
                .alignment(Alignment::Center)
                .block(block)
                .light_green();

            frame.render_widget(paragraph, area[i]);
        }
    }

    fn render_chart<'a>(
        &self,
        name: &'static str,
        x_axis_title: &'static str,
        y_axis_title: &'static str,
        style: Style,
        data: &'a Vec<(f64, f64)>,
    ) -> Chart<'a> {
        // Create the datasets to fill the chart with
        let datasets = vec![
            // Line chart
            Dataset::default()
                .name(name)
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(style)
                .data(data),
        ];

        let min_value_x = data
            .iter()
            .map(|&(x, _)| x)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let max_value_x = data
            .iter()
            .map(|&(x, _)| x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        // Create the X axis and define its properties
        let x_axis = Axis::default()
            .title(x_axis_title.green())
            .style(Style::default().green())
            .bounds([min_value_x, max_value_x])
            .labels(vec![
                min_value_x.to_string().into(),
                ((min_value_x + max_value_x) / 2.0)
                    .round()
                    .to_string()
                    .into(),
                max_value_x.to_string().into(),
            ]);

        let min_value_y = data
            .iter()
            .map(|&(_, y)| y)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let max_value_y = data
            .iter()
            .map(|&(_, y)| y)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        // Create the Y axis and define its properties
        let y_axis = Axis::default()
            .title(y_axis_title.green())
            .style(Style::default().green())
            .bounds([
                min_value_y - (min_value_y * 0.1),
                max_value_y + (max_value_y * 0.1),
            ])
            .labels(vec![
                ((((min_value_y - (min_value_y * 0.1)) * 100.0).round()) / 100.0)
                    .to_string()
                    .into(),
                (((((min_value_y + max_value_y) / 2.0) * 100.0).round()) / 100.0)
                    .to_string()
                    .into(),
                ((((max_value_y + (max_value_y * 0.1)) * 100.0).round()) / 100.0)
                    .to_string()
                    .into(),
            ]);

        // Create the chart and link all the parts together
        Chart::new(datasets)
            .block(Block::new())
            .x_axis(x_axis)
            .y_axis(y_axis)
    }

    fn render(
        &self,
        frame: &mut Frame,
        layout: Rect,
        stats_title_left: Vec<&str>,
        stats_title_right: Vec<&str>,
        stats_value_left: Vec<&str>,
        stats_value_right: Vec<&str>,
        chart_name: &'static str,
        chart_x_axis_title: &'static str,
        chart_y_axis_title: &'static str,
        chart_data: Vec<(f64, f64)>,
    ) {
        // Rendering Stats
        let layout_1 = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .margin(1)
        .split(layout);

        // Split the area in 2 segments:
        let stats_layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .margin(1)
        .split(layout_1[0]);

        let stats_left = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ],
        )
        .split(stats_layout[0]);

        let stats_right = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ],
        )
        .split(stats_layout[1]);

        self.render_stats(frame, stats_left, stats_title_left, stats_value_left);

        self.render_stats(frame, stats_right, stats_title_right, stats_value_right);

        // Rendering Charts
        frame.render_widget(
            self.render_chart(
                chart_name,
                chart_x_axis_title,
                chart_y_axis_title,
                Style::default().white(),
                &chart_data,
            ),
            layout_1[1],
        );
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
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
