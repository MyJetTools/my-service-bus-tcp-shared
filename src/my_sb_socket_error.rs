use std::{io::Error, string::FromUtf8Error};

#[derive(Debug)]
pub enum MySbSocketError {
    SocketDisconnected,
    ErrorReadingSize,
    InvalidPacketId(u8),
    ParsingUtf8StringError(FromUtf8Error),
    IoError(Error),
}

impl From<Error> for MySbSocketError {
    fn from(src: Error) -> Self {
        Self::IoError(src)
    }
}

impl From<FromUtf8Error> for MySbSocketError {
    fn from(src: FromUtf8Error) -> Self {
        Self::ParsingUtf8StringError(src)
    }
}
