//! Write output to a file and rotate the files when limits have been exceeded.
//!
//! Defines a simple [std::io::Write] object that you can plug into your writers as middleware.
//!
//! # Rotating by Lines #
//!
//! We can rotate log files by using the amount of lines as a limit.
//!
//! ```
//! use file_rotate::{FileRotate, RotationMode};
//! use std::{fs, io::Write};
//!
//! // Create a directory to store our logs, this is not strictly needed but shows how we can
//! // arbitrary paths.
//! fs::create_dir("target/my-log-directory-lines");
//!
//! // Create a new log writer. The first argument is anything resembling a path. The
//! // basename is used for naming the log files.
//! //
//! // Here we choose to limit logs by 10 lines, and have at most 2 rotated log files. This
//! // makes the total amount of log files 4, since the original file is present as well as
//! // file 0.
//! let mut log = FileRotate::new("target/my-log-directory-lines/my-log-file", RotationMode::Lines(3), 2);
//!
//! // Write a bunch of lines
//! writeln!(log, "Line 1: Hello World!");
//! for idx in 2..11 {
//!     writeln!(log, "Line {}", idx);
//! }
//!
//! assert_eq!("Line 10\n", fs::read_to_string("target/my-log-directory-lines/my-log-file").unwrap());
//!
//! assert_eq!("Line 1: Hello World!\nLine 2\nLine 3\n", fs::read_to_string("target/my-log-directory-lines/my-log-file.0").unwrap());
//! assert_eq!("Line 4\nLine 5\nLine 6\n", fs::read_to_string("target/my-log-directory-lines/my-log-file.1").unwrap());
//! assert_eq!("Line 7\nLine 8\nLine 9\n", fs::read_to_string("target/my-log-directory-lines/my-log-file.2").unwrap());
//!
//! fs::remove_dir_all("target/my-log-directory-lines");
//! ```
//!
//! # Rotating by Bytes #
//!
//! Another method of rotation is by bytes instead of lines.
//!
//! ```
//! use file_rotate::{FileRotate, RotationMode};
//! use std::{fs, io::Write};
//!
//! fs::create_dir("target/my-log-directory-bytes");
//!
//! let mut log = FileRotate::new("target/my-log-directory-bytes/my-log-file", RotationMode::Bytes(5), 2);
//!
//! writeln!(log, "Test file");
//!
//! assert_eq!("Test ", fs::read_to_string("target/my-log-directory-bytes/my-log-file.0").unwrap());
//! assert_eq!("file\n", fs::read_to_string("target/my-log-directory-bytes/my-log-file").unwrap());
//!
//! fs::remove_dir_all("target/my-log-directory-bytes");
//! ```
//!
//! # Rotation Method #
//!
//! The rotation method used is to always write to the base path, and then move the file to a new
//! location when the limit is exceeded. The moving occurs in the sequence 0, 1, 2, n, 0, 1, 2...
//!
//! Here's an example with 1 byte limits:
//!
//! ```
//! use file_rotate::{FileRotate, RotationMode};
//! use std::{fs, io::Write};
//!
//! fs::create_dir("target/my-log-directory-small");
//!
//! let mut log = FileRotate::new("target/my-log-directory-small/my-log-file", RotationMode::Bytes(1), 3);
//!
//! write!(log, "A");
//! assert_eq!("A", fs::read_to_string("target/my-log-directory-small/my-log-file").unwrap());
//!
//! write!(log, "B");
//! assert_eq!("A", fs::read_to_string("target/my-log-directory-small/my-log-file.0").unwrap());
//! assert_eq!("B", fs::read_to_string("target/my-log-directory-small/my-log-file").unwrap());
//!
//! write!(log, "C");
//! assert_eq!("A", fs::read_to_string("target/my-log-directory-small/my-log-file.0").unwrap());
//! assert_eq!("B", fs::read_to_string("target/my-log-directory-small/my-log-file.1").unwrap());
//! assert_eq!("C", fs::read_to_string("target/my-log-directory-small/my-log-file").unwrap());
//!
//! write!(log, "D");
//! assert_eq!("A", fs::read_to_string("target/my-log-directory-small/my-log-file.0").unwrap());
//! assert_eq!("B", fs::read_to_string("target/my-log-directory-small/my-log-file.1").unwrap());
//! assert_eq!("C", fs::read_to_string("target/my-log-directory-small/my-log-file.2").unwrap());
//! assert_eq!("D", fs::read_to_string("target/my-log-directory-small/my-log-file").unwrap());
//!
//! write!(log, "E");
//! assert_eq!("A", fs::read_to_string("target/my-log-directory-small/my-log-file.0").unwrap());
//! assert_eq!("B", fs::read_to_string("target/my-log-directory-small/my-log-file.1").unwrap());
//! assert_eq!("C", fs::read_to_string("target/my-log-directory-small/my-log-file.2").unwrap());
//! assert_eq!("D", fs::read_to_string("target/my-log-directory-small/my-log-file.3").unwrap());
//! assert_eq!("E", fs::read_to_string("target/my-log-directory-small/my-log-file").unwrap());
//!
//!
//! // Here we overwrite the 0 file since we're out of log files, restarting the sequencing
//! write!(log, "F");
//! assert_eq!("E", fs::read_to_string("target/my-log-directory-small/my-log-file.0").unwrap());
//! assert_eq!("B", fs::read_to_string("target/my-log-directory-small/my-log-file.1").unwrap());
//! assert_eq!("C", fs::read_to_string("target/my-log-directory-small/my-log-file.2").unwrap());
//! assert_eq!("D", fs::read_to_string("target/my-log-directory-small/my-log-file.3").unwrap());
//! assert_eq!("F", fs::read_to_string("target/my-log-directory-small/my-log-file").unwrap());
//!
//! fs::remove_dir_all("target/my-log-directory-small");
//! ```
//!
//! # Filesystem Errors #
//!
//! If the directory containing the logs is deleted or somehow made inaccessible then the rotator
//! will simply continue operating without fault. When a rotation occurs, it attempts to open a
//! file in the directory. If it can, it will just continue logging. If it can't then the written
//! date is sent to the void.
//!
//! This logger never panics.
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]

use std::sync::Arc;
use std::sync::Mutex;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

// ---

/// Condition on which a file is rotated.
#[derive(Clone)]
pub enum RotationMode {
    /// Cut the log at the exact size in bytes.
    Bytes(usize),
    /// Cut the log file at line breaks.
    Lines(usize),
    /// Cut the log file after an elapsed duration.
    Duration(Duration),
    /// Combine multiple rotation modes.
    Combine(Vec<RotationMode>),
}

/// The main writer used for rotating logs.
#[derive(Clone)]
pub struct FileRotate {
    basename: PathBuf,
    count: Arc<Mutex<usize>>,
    file: Arc<Mutex<Option<File>>>,
    file_number: Arc<Mutex<usize>>,
    max_file_number: usize,
    mode: RotationMode,
    last_rotation: Arc<Mutex<Instant>>,
}

impl FileRotate {
    /// Create a new [FileRotate].
    ///
    /// The basename of the `path` is used to create new log files by appending an extension of the
    /// form `.N`, where N is `0..=max_file_number`.
    ///
    /// `rotation_mode` specifies the limits for rotating a file.
    ///
    /// # Panics
    ///
    /// Panics if `bytes == 0` or `lines == 0`.
    pub fn new<P: AsRef<Path>>(
        path: P,
        rotation_mode: RotationMode,
        max_file_number: usize,
    ) -> Self {
        match rotation_mode.clone() {
            RotationMode::Bytes(bytes) => {
                assert!(bytes > 0);
            }
            RotationMode::Lines(lines) => {
                assert!(lines > 0);
            }
            RotationMode::Duration(_) => {}
            RotationMode::Combine(modes) => assert!(modes.len() > 0),
        };

        let file_rotate = Self {
            basename: path.as_ref().to_path_buf(),
            count: Arc::new(Mutex::new(0)),
            file: Arc::new(Mutex::new(match File::create(&path) {
                Ok(file) => Some(file),
                Err(_) => None,
            })),
            file_number: Arc::new(Mutex::new(0)),
            max_file_number,
            mode: rotation_mode.clone(),
            last_rotation: Arc::new(Mutex::new(Instant::now())),
        };

        let file_rotate_c = file_rotate.clone();
        if let RotationMode::Duration(duration) = rotation_mode {
            std::thread::spawn(move || loop {
                if let Err(_) = file_rotate_c.rotate(false) {
                    // Handle error?
                }
                std::thread::sleep(duration);
            });
        }

        file_rotate
    }

    fn rotate(&self, force: bool) -> io::Result<()> {
        if !force {
            if let Ok(count) = self.count.lock() {
                if *count == 0 {
                    return Ok(());
                }
            }
        }
        let mut path = self.basename.clone();

        if let Ok(file_number) = self.file_number.lock() {
            path.set_extension(file_number.to_string());
        }

        if let Ok(mut file) = self.file.lock() {
            let _ = file.take();
            let _ = fs::rename(&self.basename, path);
            *file = Some(File::create(&self.basename)?);
            if let Ok(mut file_number) = self.file_number.lock() {
                *file_number = (*file_number + 1) % (self.max_file_number + 1);
            }
            if let Ok(mut count) = self.count.lock() {
                *count = 0;
            }
            if let Ok(mut last_rotation) = self.last_rotation.lock() {
                *last_rotation = Instant::now();
            }
        }

        Ok(())
    }
}

impl Write for FileRotate {
    fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
        let written = buf.len();
        match self.mode {
            RotationMode::Bytes(bytes) => {
                loop {
                    if let Ok(count) = self.count.lock() {
                        if let Ok(mut file) = self.file.lock() {
                        if *count + buf.len() > bytes {
                            let bytes_left = bytes - *count;
                                if let Some(Err(err)) =
                                    file.as_mut().map(|file| file.write(&buf[..bytes_left]))
                                {
                                    return Err(err);
                                }
                                buf = &buf[bytes_left..];
                            } else {
                                break;
                            }
                        }
                    }
                    self.rotate(true)?;
                }
                if let Ok(mut count) = self.count.lock() {
                    *count += buf.len();
                }
                if let Ok(mut file) = self.file.lock() {
                    if let Some(Err(err)) = file.as_mut().map(|file| file.write(&buf[..])) {
                        return Err(err);
                    }
                }
            }
            RotationMode::Lines(lines) => {
                let mut should_rotate = false;
                if let Ok(mut file) = self.file.lock() {
                    while let Some((idx, _)) =
                        buf.iter().enumerate().find(|(_, byte)| *byte == &b'\n')
                    {
                        if let Some(Err(err)) =
                            file.as_mut().map(|file| file.write(&buf[..idx + 1]))
                        {
                            return Err(err);
                        }
                        if let Ok(mut count) = self.count.lock() {
                            *count += 1;
                            buf = &buf[idx + 1..];
                            if *count >= lines {
                                should_rotate = true;
                            }
                        }
                    }
                    if let Some(Err(err)) = file.as_mut().map(|file| file.write(buf)) {
                        return Err(err);
                    }
                }
                if should_rotate {
                    self.rotate(false)?;
                }
            }
            RotationMode::Duration(_) => {
                if let Ok(mut file) = self.file.lock() {
                    if let Ok(mut count) = self.count.lock() {
                        *count += 1;
                    }
                    if let Some(Err(err)) = file.as_mut().map(|file| file.write(buf)) {
                        return Err(err);
                    }
                }
            }
            RotationMode::Combine(_) => unimplemented!(),
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Ok(mut file) = self.file.lock() {
            if let Some(Err(err)) = file.as_mut().map(|file| file.flush()) {
                return Err(err);
            } else {
                return Ok(());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "assertion failed: bytes > 0")]
    fn zero_bytes() {
        let mut rot = FileRotate::new("target/zero_bytes", RotationMode::Bytes(0), 0);
        writeln!(rot, "Zero").unwrap();
        assert_eq!("\n", fs::read_to_string("target/zero_bytes").unwrap());
        assert_eq!("o", fs::read_to_string("target/zero_bytes.0").unwrap());
    }

    #[test]
    #[should_panic(expected = "assertion failed: lines > 0")]
    fn zero_lines() {
        let mut rot = FileRotate::new("target/zero_lines", RotationMode::Lines(0), 0);
        write!(rot, "a\nb\nc\nd\n").unwrap();
        assert_eq!("", fs::read_to_string("target/zero_lines").unwrap());
        assert_eq!("d\n", fs::read_to_string("target/zero_lines.0").unwrap());
    }

    #[test]
    fn rotate_to_deleted_directory() {
        let _ = fs::remove_dir_all("target/rotate");
        fs::create_dir("target/rotate").unwrap();

        let mut rot = FileRotate::new("target/rotate/log", RotationMode::Lines(1), 0);
        writeln!(rot, "a").unwrap();
        assert_eq!("", fs::read_to_string("target/rotate/log").unwrap());
        assert_eq!("a\n", fs::read_to_string("target/rotate/log.0").unwrap());

        fs::remove_dir_all("target/rotate").unwrap();

        assert!(writeln!(rot, "b").is_err());

        rot.flush().unwrap();
        assert!(fs::read_dir("target/rotate").is_err());
        fs::create_dir("target/rotate").unwrap();

        writeln!(rot, "c").unwrap();
        assert_eq!("", fs::read_to_string("target/rotate/log").unwrap());

        writeln!(rot, "d").unwrap();
        assert_eq!("", fs::read_to_string("target/rotate/log").unwrap());
        assert_eq!("d\n", fs::read_to_string("target/rotate/log.0").unwrap());
    }

    #[quickcheck_macros::quickcheck]
    fn arbitrary_lines(count: usize) {
        let _ = fs::remove_dir_all("target/arbitrary_lines");
        fs::create_dir("target/arbitrary_lines").unwrap();

        let count = count.max(1);
        let mut rot = FileRotate::new("target/arbitrary_lines/log", RotationMode::Lines(count), 0);

        for _ in 0..count - 1 {
            writeln!(rot).unwrap();
        }

        rot.flush().unwrap();
        assert!(!Path::new("target/arbitrary_lines/log.0").exists());
        writeln!(rot).unwrap();
        assert!(Path::new("target/arbitrary_lines/log.0").exists());

        fs::remove_dir_all("target/arbitrary_lines").unwrap();
    }

    #[quickcheck_macros::quickcheck]
    fn arbitrary_bytes() {
        let _ = fs::remove_dir_all("target/arbitrary_bytes");
        fs::create_dir("target/arbitrary_bytes").unwrap();

        let count = 0.max(1);
        let mut rot = FileRotate::new("target/arbitrary_bytes/log", RotationMode::Bytes(count), 0);

        for _ in 0..count {
            write!(rot, "0").unwrap();
        }

        rot.flush().unwrap();
        assert!(!Path::new("target/arbitrary_bytes/log.0").exists());
        write!(rot, "1").unwrap();
        assert!(Path::new("target/arbitrary_bytes/log.0").exists());

        fs::remove_dir_all("target/arbitrary_bytes").unwrap();
    }
}
