use crate::theme::font::{FontRender, MetricSnips};

use sdl2::rect::Point;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GameMetricType {
    Score,
    Level,
    VirusCount,
}

impl GameMetricType {
    pub fn label(&self) -> &str {
        match self {
            GameMetricType::Score => "Score",
            GameMetricType::Level => "Level",
            GameMetricType::VirusCount => "Virus",
        }
    }
}

pub struct GameMetricsRow {
    metric: GameMetricType,
    value: MetricSnips,
    label: Point,
    label_width: u32,
    value_width: u32,
}

impl GameMetricsRow {
    fn width(&self) -> u32 {
        self.value_width.max(self.label_width)
    }

    pub fn metric(&self) -> GameMetricType {
        self.metric
    }
    pub fn label(&self) -> Point {
        self.label
    }
    pub fn value(&self) -> MetricSnips {
        self.value
    }
}

pub struct GameMetricsTable {
    rows: Vec<GameMetricsRow>,
}

impl GameMetricsTable {
    pub fn new(
        bottle_visible_height: u32,
        font: &FontRender,
        font_bold: &FontRender,
        labelled_max: &[(GameMetricType, u32)],
    ) -> Self {
        let mut y = bottle_visible_height as i32; // start from the bottom
        let x = 0;
        let rows = labelled_max
            .iter()
            .rev()
            .copied()
            .map(|(metric, max)| {
                let (value_width, value_height) = font.number_size(max);
                let (label_width, label_height) = font_bold.string_size(metric.label());
                y -= value_height as i32;
                let value = MetricSnips::left((x, y), max);
                y -= label_height as i32;
                let label = Point::new(x, y);
                y -= 10;
                GameMetricsRow {
                    metric,
                    value,
                    label,
                    value_width,
                    label_width,
                }
            })
            .collect();

        Self { rows }
    }

    pub fn offset_x(&mut self, x: i32) {
        for row in self.rows.iter_mut() {
            row.value = row.value.offset(x, 0);
            row.label = row.label.offset(x, 0);
        }
    }

    pub fn width(&self) -> u32 {
        self.rows.iter().map(|r| r.width()).max().unwrap()
    }

    pub fn rows(&self) -> &Vec<GameMetricsRow> {
        &self.rows
    }
}
