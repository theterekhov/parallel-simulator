use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum Strategy {
    #[default]
    CPthreads,
    PythonGil,
    GoChannels,
}

impl Strategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Strategy::CPthreads => "C (Pthreads)",
            Strategy::PythonGil => "Python (GIL)",
            Strategy::GoChannels => "Go (Channels",
        }
    }
}
