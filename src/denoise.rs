use crate::data;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn denoise(points: &[data::Point]) -> Vec<data::Point> {
    let mut out: Vec<data::Point> = vec![];

    for (index, point) in points.iter().enumerate() {
        if should_apply_point(index, point, points) {
            out.push(point.clone());
        }
    }

    out
}

fn should_apply_point(index: usize, point: &data::Point, points: &[data::Point]) -> bool {
    let mut matching = 0;
    for point_2 in points[index..].iter().chain(points[..index].iter()) {
        if point.time == point_2.time && point.height == point_2.height {
            continue;
        }

        if time_diff(&point.time, &point_2.time) > 10 {
            // Nothing within 10 seconds
            return false;
        }

        let height_delta = (i64::from(point.height) - i64::from(point_2.height)).abs();

        if height_delta < 1000 {
            // Found a matching point within 1000ft, within 10s
            matching += 1;

            if matching > 5 {
                return true;
            }
        }
    }

    false
}

fn time_diff(a: &SystemTime, b: &SystemTime) -> u64 {
    let time_a = a.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let time_b = b.duration_since(UNIX_EPOCH).unwrap().as_secs();

    time_a.abs_diff(time_b)
}
