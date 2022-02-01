use std::ffi::NulError;
use std::{error, fmt, io};

use sdl2::video::WindowBuildError;
use sdl2::IntegerOrSdlError;

#[derive(Debug)]
pub enum Error {
    GeneralError(&'static str),
    AlsaError(alsa::Error),
    IoError(io::Error),
    NulError(NulError),
    SdlIntError(IntegerOrSdlError),
    SdlStrError(String),
    SdlWindowBuildError(WindowBuildError),
    TomlSerializeError(toml::ser::Error),
    TomlDeserializeError(toml::de::Error),
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Self::AlsaError(ref err) => Some(err),
            Self::IoError(ref err) => Some(err),
            Self::NulError(ref err) => Some(err),
            Self::SdlIntError(ref err) => Some(err),
            Self::SdlWindowBuildError(ref err) => Some(err),
            Self::TomlSerializeError(ref err) => Some(err),
            Self::TomlDeserializeError(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::GeneralError(ref err) => {
                write!(f, "General error: {}", err)
            }
            Self::AlsaError(ref err) => {
                write!(f, "ALSA error: {}", err)
            }
            Self::IoError(ref err) => {
                write!(f, "I/O error: {}", err)
            }
            Self::NulError(ref err) => {
                write!(f, "Nul byte error: {}", err)
            }
            Self::SdlIntError(ref err) => {
                write!(f, "SDL error: {}", err)
            }
            Self::SdlStrError(ref err) => {
                write!(f, "SDL error: {}", err)
            }
            Self::SdlWindowBuildError(ref err) => {
                write!(f, "SDL window builder error: {}", err)
            }
            Self::TomlSerializeError(ref err) => {
                write!(f, "TOML serialization error: {}", err)
            }
            Self::TomlDeserializeError(ref err) => {
                write!(f, "TOML deserialization error: {}", err)
            }
        }
    }
}

impl From<alsa::Error> for Error {
    fn from(err: alsa::Error) -> Self {
        Self::AlsaError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<NulError> for Error {
    fn from(err: NulError) -> Self {
        Self::NulError(err)
    }
}

impl From<IntegerOrSdlError> for Error {
    fn from(err: IntegerOrSdlError) -> Self {
        Self::SdlIntError(err)
    }
}

pub fn sdl_error(err: String) -> Error {
    Error::SdlStrError(err)
}

impl From<WindowBuildError> for Error {
    fn from(err: WindowBuildError) -> Self {
        Self::SdlWindowBuildError(err)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Self::TomlSerializeError(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Self::TomlDeserializeError(err)
    }
}
