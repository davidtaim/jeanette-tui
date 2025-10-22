use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Cell, Clear, HighlightSpacing, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState,
    },
};
use std::io;
use unicode_width::UnicodeWidthStr;

use crate::nmcli_wrapper::{Network, NmcliWrapper};

const TABLE_HEADERS: [&str; 9] = [
    "IN USE", "BSSID", "SSID", "MODE", "CHAN", "RATE", "SIGNAL", "BARS", "SECURITY",
];

const ITEM_HEIGHT: usize = 1;

struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    normal_row_color: Color,
    footer_fg_color: Color,
    network_border_color: Color,
}

impl TableColors {
    const fn new() -> Self {
        Self {
            buffer_bg: Color::Reset,
            header_bg: Color::Green,
            header_fg: Color::Black,
            row_fg: Color::White,
            selected_row_style_fg: Color::Blue,
            normal_row_color: Color::Reset,
            footer_fg_color: Color::Green,
            network_border_color: Color::Blue,
        }
    }
}

pub struct JeanetteApp {
    state: TableState,
    items: Vec<Network>,
    longest_item_lens: (u16, u16, u16, u16, u16, u16, u16, u16, u16),
    scroll_state: ScrollbarState,
    colors: TableColors,
    show_connect_popup: bool,
}

impl JeanetteApp {
    pub fn new() -> Self {
        let data_vec = NmcliWrapper::get_networks_list();
        Self {
            state: TableState::default().with_selected(0),
            longest_item_lens: constraint_len_calculator(&data_vec),
            scroll_state: ScrollbarState::new(data_vec.len() - 1 * ITEM_HEIGHT),
            colors: TableColors::new(),
            items: data_vec,
            show_connect_popup: false,
        }
    }

    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new();
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('s') => self.scan_networks_list(),
                        KeyCode::Char('c') => self.show_connect_popup = true,
                        KeyCode::Esc => {
                            if self.show_connect_popup {
                                self.show_connect_popup = false;
                            }
                        }
                        KeyCode::Down => self.next_row(),
                        KeyCode::Up => self.previous_row(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = &Layout::vertical([
            Constraint::Min(5),
            Constraint::Length(7),
            Constraint::Length(1),
        ]);
        let rects = vertical.split(frame.area());

        self.set_colors();

        self.render_table(frame, rects[0]);
        self.render_scrollbar(frame, rects[0]);
        self.render_network_info(frame, rects[1]);
        self.render_footer(frame, rects[2]);

        if self.show_connect_popup {
            self.render_edit_popup(frame, frame.area());
        }
    }

    fn render_edit_popup(&mut self, frame: &mut Frame, area: Rect) {
        let connection = &self.items[self.state.selected().unwrap()];
        let block = Block::bordered()
            .title("Connect to network")
            .border_type(BorderType::Plain)
            .border_style(Style::new().fg(self.colors.network_border_color));

        let area_popup = self.popup_area(area, 80, 40);
        frame.render_widget(Clear, area_popup);
        frame.render_widget(block, area_popup);
    }

    fn popup_area(&mut self, area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

    fn scan_networks_list(&mut self) {
        self.items = NmcliWrapper::get_networks_list();
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);

        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg);

        let header = TABLE_HEADERS
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);

        let rows = self.items.iter().map(|data| {
            let item = data.ref_array();
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("{content}"))))
                .collect::<Row>()
                .style(Style::new().fg(self.colors.row_fg))
                .bg(self.colors.normal_row_color)
                .height(1)
        });

        let block_table = Block::bordered()
            .title("Network List")
            .border_type(BorderType::Plain)
            .border_style(Style::new().fg(self.colors.network_border_color));

        let t = Table::new(
            rows,
            [
                Constraint::Max(self.longest_item_lens.0),
                Constraint::Min(self.longest_item_lens.1),
                Constraint::Min(self.longest_item_lens.2),
                Constraint::Min(self.longest_item_lens.3),
                Constraint::Min(self.longest_item_lens.4),
                Constraint::Min(self.longest_item_lens.5),
                Constraint::Min(self.longest_item_lens.6),
                Constraint::Min(self.longest_item_lens.7),
                Constraint::Min(self.longest_item_lens.8),
            ],
        )
        .block(block_table)
        .header(header)
        .row_highlight_style(selected_row_style)
        .bg(self.colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut self.state);
    }

    fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }

    fn render_network_info(&self, frame: &mut Frame, area: Rect) {
        let device_name = NmcliWrapper::get_device_name();
        let network_info = NmcliWrapper::get_device_info(device_name);

        let lines = vec![
            Line::from(vec!["Device: ".cyan(), network_info.device.green()]),
            Line::from(vec!["Connection: ".cyan(), network_info.connection.green()]),
            Line::from(vec!["IP: ".cyan(), network_info.ip4_address.green()]),
            Line::from(vec!["Gateway: ".cyan(), network_info.ip4_gateway.green()]),
            Line::from(vec!["DNS: ".cyan(), network_info.ip4_dns.green()]),
        ];

        let paragraph_network_info = Paragraph::new(Text::from(lines)).block(
            Block::bordered()
                .title("Network Info")
                .border_type(BorderType::Plain)
                .border_style(Style::new().fg(self.colors.network_border_color)),
        );

        frame.render_widget(paragraph_network_info, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let info_footer = Paragraph::new(Text::from("(Q) quit | (↑) move up | (↓) move down"))
            .style(
                Style::new()
                    .fg(self.colors.footer_fg_color)
                    .bg(self.colors.buffer_bg),
            )
            .centered();
        frame.render_widget(info_footer, area);
    }
}

fn constraint_len_calculator(items: &[Network]) -> (u16, u16, u16, u16, u16, u16, u16, u16, u16) {
    let in_use_len: u16 = 6;

    let bssid_len = items
        .iter()
        .map(Network::bssid)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let ssid_len = items
        .iter()
        .map(Network::ssid)
        .flat_map(str::lines)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let mode_len = items
        .iter()
        .map(Network::mode)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let chan_len = items
        .iter()
        .map(Network::chan)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let rate_len = items
        .iter()
        .map(Network::rate)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let signal_len = items
        .iter()
        .map(Network::signal)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let bars_len = items
        .iter()
        .map(Network::bars)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let security_len = items
        .iter()
        .map(Network::security)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    #[allow(clippy::cast_possible_truncation)]
    (
        in_use_len,
        bssid_len as u16,
        ssid_len as u16,
        mode_len as u16,
        chan_len as u16,
        rate_len as u16,
        signal_len as u16,
        bars_len as u16,
        security_len as u16,
    )
}
