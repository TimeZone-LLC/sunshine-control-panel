use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub const MAGIC: &str = "SUNSHINE_STYLUS_DAT\t1";
pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_SAMPLES: usize = 200_000;

pub const EVENT_HOVER: u8 = 0;
pub const EVENT_DOWN: u8 = 1;
pub const EVENT_UP: u8 = 2;
pub const EVENT_MOVE: u8 = 3;
pub const EVENT_CANCEL: u8 = 4;
pub const EVENT_BUTTON_ONLY: u8 = 5;
pub const EVENT_HOVER_LEAVE: u8 = 6;
pub const EVENT_CANCEL_ALL: u8 = 7;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StylusSample {
    pub timestamp_us: u64,
    pub event_type: u8,
    pub x: f64,
    pub y: f64,
    pub pressure: f64,
    pub rotation: u32,
    pub tilt_x: i32,
    pub tilt_y: i32,
}

#[derive(Debug, Default)]
pub struct StylusData {
    pub samples: Vec<StylusSample>,
    pub truncated: bool,
}

fn validate_sample(sample: &StylusSample) -> bool {
    matches!(
        sample.event_type,
        EVENT_HOVER
            | EVENT_DOWN
            | EVENT_UP
            | EVENT_MOVE
            | EVENT_CANCEL
            | EVENT_BUTTON_ONLY
            | EVENT_HOVER_LEAVE
            | EVENT_CANCEL_ALL
    ) && sample.x.is_finite()
        && (0.0..=1.0).contains(&sample.x)
        && sample.y.is_finite()
        && (0.0..=1.0).contains(&sample.y)
        && sample.pressure.is_finite()
        && (0.0..=1.0).contains(&sample.pressure)
        && ((-90..=90).contains(&sample.tilt_x) || sample.tilt_x == 0xff)
        && ((-90..=90).contains(&sample.tilt_y) || sample.tilt_y == 0xff)
}

pub fn write_header(writer: &mut impl Write) -> std::io::Result<()> {
    writeln!(writer, "{MAGIC}")?;
    writeln!(
        writer,
        "# columns=P timestamp_us event_type x y pressure rotation tilt_x tilt_y"
    )
}

pub fn write_sample(writer: &mut impl Write, sample: &StylusSample) -> std::io::Result<()> {
    writeln!(
        writer,
        "P {} {} {:.17} {:.17} {:.17} {} {} {}",
        sample.timestamp_us,
        sample.event_type,
        sample.x,
        sample.y,
        sample.pressure,
        sample.rotation,
        sample.tilt_x,
        sample.tilt_y,
    )
}

pub fn load(path: &Path) -> Result<StylusData, String> {
    let metadata =
        std::fs::metadata(path).map_err(|_| "Unable to read the data file.".to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err("The data file is invalid or exceeds 64 MiB.".to_string());
    }

    let file = File::open(path).map_err(|_| "Unable to open the data file.".to_string())?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    reader
        .read_line(&mut first_line)
        .map_err(|_| "Unable to read the data file.".to_string())?;
    if first_line.trim_end_matches(['\r', '\n']) != MAGIC {
        return Err("The data file format or version is not supported.".to_string());
    }

    let mut data = StylusData::default();
    let mut previous_timestamp = None;
    let mut line = String::new();
    loop {
        line.clear();
        let length = reader
            .read_line(&mut line)
            .map_err(|_| "Unable to read the data file.".to_string())?;
        if length == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') {
            data.truncated |= trimmed == "# truncated=true";
            continue;
        }

        let fields = trimmed.split_whitespace().collect::<Vec<_>>();
        if fields.len() != 9 || fields[0] != "P" {
            return Err("The data file contains an invalid record.".to_string());
        }
        let sample = StylusSample {
            timestamp_us: fields[1]
                .parse()
                .map_err(|_| "The data file contains an invalid timestamp.".to_string())?,
            event_type: fields[2]
                .parse()
                .map_err(|_| "The data file contains an invalid event.".to_string())?,
            x: fields[3]
                .parse()
                .map_err(|_| "The data file contains an invalid coordinate.".to_string())?,
            y: fields[4]
                .parse()
                .map_err(|_| "The data file contains an invalid coordinate.".to_string())?,
            pressure: fields[5]
                .parse()
                .map_err(|_| "The data file contains an invalid pressure value.".to_string())?,
            rotation: fields[6]
                .parse()
                .map_err(|_| "The data file contains an invalid rotation value.".to_string())?,
            tilt_x: fields[7]
                .parse()
                .map_err(|_| "The data file contains an invalid tilt value.".to_string())?,
            tilt_y: fields[8]
                .parse()
                .map_err(|_| "The data file contains an invalid tilt value.".to_string())?,
        };
        if !validate_sample(&sample) {
            return Err("A sample in the data file is outside the allowed range.".to_string());
        }
        if previous_timestamp.is_some_and(|previous| sample.timestamp_us < previous) {
            return Err("The timestamps in the data file are not monotonic.".to_string());
        }
        if data.samples.len() == MAX_SAMPLES {
            return Err("The data file exceeds the 200000-sample import limit.".to_string());
        }
        previous_timestamp = Some(sample.timestamp_us);
        data.samples.push(sample);
    }
    if data.samples.is_empty() {
        return Err("The data file contains no stylus samples.".to_string());
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read};

    fn test_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "alkaidlab-stylus-{name}-{}-{}.dat",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn sample_round_trip_preserves_values() {
        let sample = StylusSample {
            timestamp_us: 12_345,
            event_type: EVENT_MOVE,
            x: 0.25,
            y: 0.75,
            pressure: 0.5,
            rotation: 270,
            tilt_x: 42,
            tilt_y: -17,
        };
        let mut bytes = Vec::new();
        write_header(&mut bytes).unwrap();
        write_sample(&mut bytes, &sample).unwrap();

        let mut reader = Cursor::new(bytes);
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();
        let fields = text
            .lines()
            .last()
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>();
        assert_eq!(fields[0], "P");
        assert_eq!(fields[1], "12345");
        assert_eq!(fields[2], "3");
        assert_eq!(fields[6], "270");
        assert_eq!(fields[7], "42");
        assert_eq!(fields[8], "-17");
    }

    #[test]
    fn validation_rejects_non_finite_coordinates() {
        let sample = StylusSample {
            timestamp_us: 0,
            event_type: EVENT_MOVE,
            x: f64::NAN,
            y: 0.5,
            pressure: 0.5,
            rotation: 0,
            tilt_x: 0,
            tilt_y: 0,
        };
        assert!(!validate_sample(&sample));
    }

    #[test]
    fn loader_rejects_decreasing_timestamps() {
        let path = test_path("timestamps");
        std::fs::write(
            &path,
            format!("{MAGIC}\nP 10 1 0.1 0.1 0.5 0 0 0\nP 9 3 0.2 0.2 0.5 0 0 0\n"),
        )
        .unwrap();
        let error = load(&path).unwrap_err();
        let _ = std::fs::remove_file(path);
        assert!(error.contains("timestamp"));
    }

    #[test]
    fn loader_preserves_asymmetric_tilt_values() {
        let path = test_path("tilt-round-trip");
        let sample = StylusSample {
            timestamp_us: 12_345,
            event_type: EVENT_MOVE,
            x: 0.25,
            y: 0.75,
            pressure: 0.5,
            rotation: 270,
            tilt_x: 42,
            tilt_y: -17,
        };
        let mut bytes = Vec::new();
        write_header(&mut bytes).unwrap();
        write_sample(&mut bytes, &sample).unwrap();
        std::fs::write(&path, bytes).unwrap();

        let loaded = load(&path).unwrap();
        let _ = std::fs::remove_file(path);
        assert_eq!(loaded.samples, vec![sample]);
    }

    #[test]
    fn loader_accepts_equal_timestamps() {
        let path = test_path("equal-timestamps");
        std::fs::write(
            &path,
            format!("{MAGIC}\nP 10 1 0.1 0.1 0.5 0 0 0\nP 10 3 0.2 0.2 0.5 0 0 0\n"),
        )
        .unwrap();

        let loaded = load(&path).unwrap();
        let _ = std::fs::remove_file(path);
        assert_eq!(loaded.samples.len(), 2);
    }

    #[test]
    fn loader_rejects_files_without_samples() {
        let path = test_path("empty");
        std::fs::write(&path, format!("{MAGIC}\n# empty\n")).unwrap();

        let error = load(&path).unwrap_err();
        let _ = std::fs::remove_file(path);
        assert!(error.contains("no stylus samples"));
    }
}
