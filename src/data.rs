use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
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
    let file_writer = BufWriter::new(file);
    let mut writer = ZlibEncoder::new(file_writer, Compression::fast());

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

    writer.flush()?;

    Ok(())
}

pub fn read(path: &str) -> Result<Plot, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut file_reader = BufReader::new(file);

    // Read the first byte of the file
    let mut first_byte = [0; 1];
    file_reader.read_exact(&mut first_byte)?;
    file_reader.seek(SeekFrom::Start(0))?;

    let mut reader: Box<dyn Read> = if first_byte[0] == HEADER[0] {
        Box::new(file_reader)
    } else {
        Box::new(ZlibDecoder::new(file_reader))
    };

    let mut byte_header = [0; 4];
    reader.read_exact(&mut byte_header)?;

    if byte_header != HEADER {
        return Err("Unexpected file header".into());
    }

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
