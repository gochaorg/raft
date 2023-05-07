use logs::{logfile::LogErr, bbuff::absbuff::ABuffError};

#[derive(Debug,Clone)]
pub enum LogToolErr {
    Log(LogErr),
    BuffErr(ABuffError),
    IOError { message:String, os_error:Option<i32> },
    FileSizeToBig
}

impl From<ABuffError> for LogToolErr {
    fn from(value: ABuffError) -> Self {
        Self::BuffErr(value.clone())
    }
}

impl From<std::io::Error> for LogToolErr {
    fn from(value: std::io::Error) -> Self {
        Self::IOError { message: value.to_string(), os_error: value.raw_os_error() }
    }
}

impl From<LogErr> for LogToolErr {
    fn from(value: LogErr) -> Self {
        Self::Log(value.clone())
    }
}
