#[cfg(feature = "bincode")]
use bincode::error::EncodeError;
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[cfg_attr(
    any(feature = "bincode", feature = "json"),
    derive(Deserialize, Serialize)
)]
#[derive(Debug, Clone, PartialEq)]
pub struct Statistics {
    pub dirs: i32,
    pub files: i32,
    pub slinks: i32,
    pub hlinks: i32,
    pub devices: i32,
    pub pipes: i32,
    pub size: u64,
    pub usage: u64,
    pub errors: Vec<String>,
    pub duration: f64,
}

impl Statistics {
    pub fn new() -> Self {
        Statistics {
            dirs: 0,
            files: 0,
            slinks: 0,
            hlinks: 0,
            devices: 0,
            pipes: 0,
            size: 0,
            usage: 0,
            errors: Vec::new(),
            duration: 0.0,
        }
    }

    pub fn clear(&mut self) {
        self.dirs = 0;
        self.files = 0;
        self.slinks = 0;
        self.hlinks = 0;
        self.devices = 0;
        self.pipes = 0;
        self.size = 0;
        self.usage = 0;
        self.errors.clear();
        self.duration = 0.0;
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    #[cfg(feature = "bincode")]
    pub fn to_vec(&self) -> Result<Vec<u8>, EncodeError> {
        bincode::serde::encode_to_vec(self, bincode::config::legacy())
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Self::new()
    }
}
