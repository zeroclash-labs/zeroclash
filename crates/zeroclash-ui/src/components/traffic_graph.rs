use gpui::{FontWeight, SharedString, div, prelude::*, px};

use crate::design::{Colors, SPACE_SM, SPACE_XL, SPACE_XS};

#[derive(Default)]
pub struct TrafficHistory {
    pub upload: Vec<f64>,
    pub download: Vec<f64>,
    pub max_upload: f64,
    pub max_download: f64,
}

impl TrafficHistory {
    pub fn push(&mut self, up: u64, down: u64) {
        let up = up as f64;
        let down = down as f64;
        self.upload.push(up);
        self.download.push(down);
        if up > self.max_upload {
            self.max_upload = up;
        }
        if down > self.max_download {
            self.max_download = down;
        }
        if self.upload.len() > 60 {
            self.upload.remove(0);
            self.download.remove(0);
        }
    }
}

pub fn traffic_summary(traffic: &TrafficHistory, c: Colors) -> impl IntoElement {
    let up_speed = traffic.upload.last().copied().unwrap_or(0.0);
    let down_speed = traffic.download.last().copied().unwrap_or(0.0);

    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_SM))
        .child(
            div()
                .flex()
                .gap(px(SPACE_XL))
                .child(speed_label("Upload", up_speed, c.accent, c))
                .child(speed_label("Download", down_speed, c.success, c)),
        )
        .child(traffic_bars(traffic, c))
}

fn speed_label(label: &str, value: f64, color: gpui::Hsla, c: Colors) -> impl IntoElement {
    div().flex().flex_col().child(
        div()
            .flex()
            .items_baseline()
            .gap(px(SPACE_XS))
            .child(div().text_color(c.text_muted).child(SharedString::from(label)))
            .child(
                div()
                    .text_color(color)
                    .font_weight(FontWeight::BOLD)
                    .child(SharedString::from(format_speed(value))),
            ),
    )
}

fn traffic_bars(traffic: &TrafficHistory, c: Colors) -> impl IntoElement {
    let max = traffic.max_upload.max(traffic.max_download).max(1.0);
    let bar_h: f32 = 40.0;

    div()
        .flex()
        .gap(px(1.0))
        .h(px(bar_h))
        .items_end()
        .children(traffic.upload.iter().enumerate().map(|(i, &up)| {
            let down = traffic.download.get(i).copied().unwrap_or(0.0);
            let up_h = (up / max * bar_h as f64).max(1.0) as f32;
            let down_h = (down / max * bar_h as f64).max(1.0) as f32;
            div()
                .flex()
                .flex_col()
                .justify_end()
                .w(px(4.0))
                .child(div().h(px(up_h)).bg(c.accent))
                .child(div().h(px(down_h)).bg(c.success))
        }))
}

pub fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_000_000.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1_000_000.0)
    } else if bytes_per_sec >= 1_000.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1_000.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}
