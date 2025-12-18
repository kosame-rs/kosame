use std::time::Duration;

#[derive(Debug, Default, Clone, Copy)]
pub struct StandardPoolConfig {
    pub max_size: usize,
    pub timeouts: StandardPoolTimeouts,
    pub queue_mode: StandardPoolQueueMode,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StandardPoolTimeouts {
    pub wait: Option<Duration>,
    pub create: Option<Duration>,
    pub recycle: Option<Duration>,
}

impl StandardPoolTimeouts {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            create: None,
            wait: None,
            recycle: None,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum StandardPoolQueueMode {
    #[default]
    Fifo,
    Lifo,
}

impl From<StandardPoolConfig> for deadpool::managed::PoolConfig {
    fn from(value: StandardPoolConfig) -> Self {
        Self {
            max_size: value.max_size,
            timeouts: value.timeouts.into(),
            queue_mode: value.queue_mode.into(),
        }
    }
}

impl From<StandardPoolTimeouts> for deadpool::managed::Timeouts {
    fn from(value: StandardPoolTimeouts) -> Self {
        Self {
            wait: value.wait,
            create: value.create,
            recycle: value.recycle,
        }
    }
}

impl From<StandardPoolQueueMode> for deadpool::managed::QueueMode {
    fn from(value: StandardPoolQueueMode) -> Self {
        match value {
            StandardPoolQueueMode::Fifo => Self::Fifo,
            StandardPoolQueueMode::Lifo => Self::Lifo,
        }
    }
}
