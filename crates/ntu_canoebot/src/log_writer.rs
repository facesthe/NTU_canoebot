use std::{
    fs::File,
    io::{BufWriter, Stderr, Write},
};

pub struct LogWriter {
    std_err: Stderr,
    file: BufWriter<File>,
}

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.std_err.write(buf)?;
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.std_err.flush()?;
        self.file.flush()
    }
}

impl LogWriter {
    /// Create a new log writer that writes any input to stderr and a file.
    pub fn to_file(file: File) -> Self {
        Self {
            std_err: std::io::stderr(),
            file: BufWriter::new(file),
        }
    }
}
