/// How a native operation interacts with the active VM lane.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NativeBehavior {
    /// Completes during the current VM instruction.
    #[default]
    Immediate,
    /// Blocks the active VM lane until the host call returns.
    Blocking,
    /// Can return a pending request and resume the task later.
    Suspending,
}

impl NativeBehavior {
    /// Stable artifact encoding.
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Immediate => 0,
            Self::Blocking => 1,
            Self::Suspending => 2,
        }
    }

    /// Decode the stable artifact representation.
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::Immediate),
            1 => Some(Self::Blocking),
            2 => Some(Self::Suspending),
            _ => None,
        }
    }
}

/// How cancellation affects a native operation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CancellationPolicy {
    /// The operation has no cancellation point once invoked.
    #[default]
    None,
    /// A queued operation can be removed before host work begins.
    BeforeStart,
    /// The operation cooperates with cancellation while in progress.
    Cooperative,
}

impl CancellationPolicy {
    /// Stable artifact encoding.
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::None => 0,
            Self::BeforeStart => 1,
            Self::Cooperative => 2,
        }
    }

    /// Decode the stable artifact representation.
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::None),
            1 => Some(Self::BeforeStart),
            2 => Some(Self::Cooperative),
            _ => None,
        }
    }
}

/// Opaque host resource category produced or consumed by a native.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    /// Open file.
    File,
    /// Network listener.
    Listener,
    /// Network connection.
    Connection,
    /// Bounded in-memory channel.
    Channel,
    /// Scope-bound task handle.
    Task,
}

impl ResourceKind {
    /// Stable artifact encoding.
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::File => 0,
            Self::Listener => 1,
            Self::Connection => 2,
            Self::Channel => 3,
            Self::Task => 4,
        }
    }

    /// Decode the stable artifact representation.
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::File),
            1 => Some(Self::Listener),
            2 => Some(Self::Connection),
            3 => Some(Self::Channel),
            4 => Some(Self::Task),
            _ => None,
        }
    }
}

/// Resource ownership effect of a native call.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ResourceEffect {
    /// Does not create or consume an owned resource.
    #[default]
    None,
    /// Creates a new owned resource.
    Creates(ResourceKind),
    /// Consumes an owned resource.
    Consumes(ResourceKind),
    /// Temporarily borrows an owned resource for one suspending operation.
    Borrows(ResourceKind),
    /// Creates one resource while temporarily borrowing another, as with
    /// accepting a connection from a listener.
    CreatesAndBorrows {
        /// Resource returned by the operation.
        created: ResourceKind,
        /// Existing resource held exclusively until the operation completes.
        borrowed: ResourceKind,
    },
}

impl ResourceEffect {
    /// Stable `(operation, kind)` artifact encoding. Kind is zero for `None`.
    pub const fn to_bytes(self) -> (u8, u8) {
        match self {
            Self::None => (0, 0),
            Self::Creates(kind) => (1, kind.to_byte()),
            Self::Consumes(kind) => (2, kind.to_byte()),
            Self::Borrows(kind) => (3, kind.to_byte()),
            Self::CreatesAndBorrows { created, borrowed } => {
                (4, (created.to_byte() << 4) | borrowed.to_byte())
            }
        }
    }

    /// Decode the stable artifact representation.
    pub const fn from_bytes(operation: u8, kind: u8) -> Option<Self> {
        match operation {
            0 if kind == 0 => Some(Self::None),
            1 => match ResourceKind::from_byte(kind) {
                Some(kind) => Some(Self::Creates(kind)),
                None => None,
            },
            2 => match ResourceKind::from_byte(kind) {
                Some(kind) => Some(Self::Consumes(kind)),
                None => None,
            },
            3 => match ResourceKind::from_byte(kind) {
                Some(kind) => Some(Self::Borrows(kind)),
                None => None,
            },
            4 => match (
                ResourceKind::from_byte(kind >> 4),
                ResourceKind::from_byte(kind & 0x0f),
            ) {
                (Some(created), Some(borrowed)) => {
                    Some(Self::CreatesAndBorrows { created, borrowed })
                }
                _ => None,
            },
            _ => None,
        }
    }
}
