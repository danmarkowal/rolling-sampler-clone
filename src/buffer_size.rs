use std::{fmt::Display, sync::Arc};

use crossbeam::atomic::AtomicCell;
use nih_plug::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Enum, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum BufferSizeUnit {
    #[default]
    #[serde(rename = "seconds")]
    Seconds,
    #[serde(rename = "notes")]
    Notes,
}

impl Display for BufferSizeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferSizeUnit::Seconds => write!(f, "Seconds"),
            BufferSizeUnit::Notes => write!(f, "Notes"),
        }
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Note(pub u32, pub u32);

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.1 == 1  {
            write!(f, "{}", self.0)
        } else {
            write!(f, "{}/{}", self.0, self.1)
        }
    }
}

#[derive(Params)]
pub(crate) struct BufferSize {
    #[persist = "unit"]
    pub unit: Arc<AtomicCell<BufferSizeUnit>>,

    #[persist = "seconds"]
    pub seconds: Arc<AtomicF32>,

    #[persist = "notes"]
    pub notes: Arc<AtomicCell<Note>>,
}
