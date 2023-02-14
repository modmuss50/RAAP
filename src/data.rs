use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const HEADER: &[u8] = &[0xd, 0x1, 0xa, 0x0];

#[derive(Clone)]
pub struct Point {
    pub height: u32,
    pub time: SystemTime,
}

#[derive(Clone)]
pub struct Plot {
    pub points: Vec<Point>,
}

pub fn write(path: &str, plot: Plot) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Header
    writer.write_all(HEADER)?;

    // Number of points
    let size = u32::try_from(plot.points.len())?;
    writer.write_all(&size.to_be_bytes())?;

    for point in plot.points {
        writer.write_all(&point.height.to_be_bytes())?;
        let time = point.time.duration_since(UNIX_EPOCH)?.as_millis();
        writer.write_all(&time.to_be_bytes())?;
    }

    Ok(())
}

pub fn read(path: &str) -> Result<Plot, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut file_header = [0; 4];
    reader.read_exact(&mut file_header)?;

    let mut size_buf = [0; 4];
    reader.read_exact(&mut size_buf)?;
    let size = u32::from_be_bytes(size_buf);

    let mut points: Vec<Point> = vec![];
    for _ in 0..size {
        let mut height_buf = [0; 4];
        reader.read_exact(&mut height_buf)?;
        let height = u32::from_be_bytes(height_buf);

        let mut time_buf = [0; 16];
        reader.read_exact(&mut time_buf)?;
        let epoch = u128::from_be_bytes(time_buf);
        let epoch64 = u64::try_from(epoch)?;
        let time = UNIX_EPOCH
            .checked_add(Duration::from_millis(epoch64))
            .ok_or("Failed to parse time")?;

        points.push(Point { height, time });
    }

    Ok(Plot { points })
}
