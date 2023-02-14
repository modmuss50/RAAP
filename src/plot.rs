use crate::{data, settings};
use eframe::egui;
use eframe::egui::plot::{MarkerShape, Plot, PlotPoints, Points};
use eframe::egui::Color32;
use std::ops::RangeInclusive;
use std::time::SystemTime;
use thousands::Separable;

pub fn plot(
    ui: &mut egui::Ui,
    points: &[data::Point],
    settings: &settings::Settings,
    data_x_age: &SystemTime,
) {
    let y_fmt = |y, _range: &RangeInclusive<f64>| {
        let ft: f64 = y;
        format!("{:}ft", ft.separate_with_commas())
    };

    Plot::new("Main plot")
        .include_y(settings.min_display_height)
        .include_y(settings.max_display_height)
        .include_x(-f64::from(settings.max_display_age))
        .include_x(0)
        .y_axis_formatter(y_fmt)
        .show_x(false)
        .show_y(false)
        .show_axes([false, settings.show_axis])
        .allow_drag(false)
        .allow_scroll(false)
        .allow_zoom(false)
        .show(ui, |plot_ui| {
            let mut series: Vec<[f64; 2]> = vec![];
            for point in points.iter().rev() {
                let seconds_ago = match data_x_age.duration_since(point.time) {
                    Ok(n) => u32::try_from(n.as_millis() / 1000).unwrap(),
                    Err(_) => continue, // Scrolled out out of view
                };

                if seconds_ago > settings.max_display_age {
                    break;
                }

                if point.height > settings.max_display_height {
                    continue;
                }

                series.push([-f64::from(seconds_ago), f64::from(point.height)]);
            }

            let points = Points::new(PlotPoints::new(series))
                .radius(1.0)
                .shape(MarkerShape::Circle)
                .color(Color32::from_rgb(100, 200, 100));
            plot_ui.points(points);
        });
}
