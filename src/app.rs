use std::{io, rc::Rc, thread::sleep, time::Duration, vec};

use crate::{database::get_network_hashrate, tui};
use crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, symbols::border, widgets::*};
#[derive(Debug, Default)]
struct NetworkStats {
    hashrate: Vec<(f64, f64)>,
    difficulty: f64,
    height: u32,
    reward: u8,
    reward_reduction: u8,
    price: u8,
}

#[derive(Debug, Default)]
struct PoolStats {
    hashrate: Vec<(f64, f64)>,
    connected_miners: u32,
    effort: f32,
    total_blocks: u32,
    block_found_time: u8,
}

#[derive(Debug, Default)]
struct MinerStats {
    hashrate: Vec<(f64, f64)>,
    average_hashrate: f64,
    pending_shares: f64,
    pending_balance: f64,
    round_contribution: f64,
    total_paid: f64,
}

#[derive(Debug, Default)]
pub struct App {
    address: String,
    network: NetworkStats,
    pool: PoolStats,
    miner: MinerStats,
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            if poll(Duration::from_millis(1000))? {
                self.handle_events()?;
            }
            self.get_hashrate();
            terminal.draw(|frame| self.render_frame(frame))?;
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
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
                (self.network.hashrate.last().unwrap().1.to_string() + " Th/s").as_str(),
                self.network.difficulty.to_string().as_str(),
                self.network.height.to_string().as_str(),
            ],
            vec![
                self.network.reward.to_string().as_str(),
                self.network.reward_reduction.to_string().as_str(),
                self.network.price.to_string().as_str(),
            ],
            "Network Hashrate",
            "Block",
            "Th/s",
            self.network.hashrate.clone(),
        );

        self.render(
            frame,
            stats_layout[1],
            vec![" Pool Hashrate ", " Connected Miners ", " Current Effort "],
            vec![" Block found every ", " Blocks found ", ""],
            vec![
                (self.pool.hashrate.last().unwrap().1.to_string() + " Gh/s").as_str(),
                self.pool.connected_miners.to_string().as_str(),
                self.pool.effort.to_string().as_str(),
            ],
            vec![
                self.pool.block_found_time.to_string().as_str(),
                self.pool.total_blocks.to_string().as_str(),
                "1",
            ],
            "Pool Hashrate",
            "Block",
            "Gh/s",
            self.pool.hashrate.clone(),
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
                (self.miner.hashrate.last().unwrap().1.to_string() + " Mh/s").as_str(),
                self.miner.average_hashrate.to_string().as_str(),
                self.miner.round_contribution.to_string().as_str(),
            ],
            vec![
                self.miner.pending_shares.to_string().as_str(),
                self.miner.pending_balance.to_string().as_str(),
                self.miner.total_paid.to_string().as_str(),
            ],
            "Miner Hashrate",
            "Time",
            "Mh/s",
            self.miner.hashrate.clone(),
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
                .marker(symbols::Marker::HalfBlock)
                .graph_type(GraphType::Line)
                .style(style)
                .data(data),
        ];

        let min_value_x = data
            .iter()
            .map(|&(x, _)| x)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let max_value_x = data
            .iter()
            .map(|&(x, _)| x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        // Create the X axis and define its properties
        let x_axis = Axis::default()
            .title(x_axis_title.green())
            .style(Style::default().green())
            .bounds([min_value_x, max_value_x])
            .labels(vec![
                min_value_x.to_string().into(),
                ((min_value_x + max_value_x) / 2.0).to_string().into(),
                max_value_x.to_string().into(),
            ]);

        let min_value_y = data
            .iter()
            .map(|&(_, y)| y)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let max_value_y = data
            .iter()
            .map(|&(_, y)| y)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        // Create the Y axis and define its properties
        let y_axis = Axis::default()
            .title(y_axis_title.green())
            .style(Style::default().green())
            .bounds([
                min_value_y - (min_value_y * 0.1),
                max_value_y + (max_value_y * 0.1),
            ])
            .labels(vec![
                (min_value_y - (min_value_y * 0.1)).to_string().into(),
                ((min_value_y + max_value_y) / 2.0).to_string().into(),
                (max_value_y + (max_value_y * 0.1)).to_string().into(),
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

    fn get_hashrate(&mut self) {
        self.network.hashrate = get_network_hashrate();
        self.pool.hashrate = get_network_hashrate();
        self.miner.hashrate = get_network_hashrate();
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
