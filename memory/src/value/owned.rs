use super::{Value, TAG_FUNCTION, TAG_SHIFT};

/// Subtype marker inside the otherwise heap-backed Function tag. Function
/// handles are canonical u32 values, so bit 59 is available for VM-owned task
/// ids without widening the public 64-bit value ABI.
pub(super) const TASK_MARKER: u64 = 1u64 << 59;
pub(super) const RESOURCE_MARKER: u64 = 1u64 << 58;
const RESOURCE_KIND_SHIFT: u32 = 56;
pub(super) const RESOURCE_KIND_MASK: u64 = 0b11u64 << RESOURCE_KIND_SHIFT;

/// Opaque host-resource class encoded without exposing a forgeable integer
/// representation to Achronyme source programs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ValueResourceKind {
    File = 0,
    Listener = 1,
    Connection = 2,
    Channel = 3,
}

impl ValueResourceKind {
    fn from_byte(value: u8) -> Self {
        match value {
            0 => Self::File,
            1 => Self::Listener,
            2 => Self::Connection,
            _ => Self::Channel,
        }
    }
}

impl Value {
    /// Create an opaque VM task handle. Task ids are not heap handles and are
    /// intentionally unavailable through [`Value::as_handle`].
    #[inline]
    pub fn task(handle: u32) -> Self {
        Value((TAG_FUNCTION << TAG_SHIFT) | TASK_MARKER | handle as u64)
    }

    /// Create an opaque VM-owned resource handle.
    #[inline]
    pub fn resource(kind: ValueResourceKind, handle: u32) -> Self {
        Value(
            (TAG_FUNCTION << TAG_SHIFT)
                | RESOURCE_MARKER
                | ((kind as u64) << RESOURCE_KIND_SHIFT)
                | handle as u64,
        )
    }

    #[inline]
    pub fn is_task(&self) -> bool {
        self.tag() == TAG_FUNCTION && self.0 & TASK_MARKER != 0
    }

    #[inline]
    pub fn is_resource(&self) -> bool {
        self.tag() == TAG_FUNCTION && self.0 & TASK_MARKER == 0 && self.0 & RESOURCE_MARKER != 0
    }

    #[inline]
    pub fn as_task_handle(&self) -> Option<u32> {
        self.is_task().then_some((self.0 & u32::MAX as u64) as u32)
    }

    #[inline]
    pub fn as_resource_handle(&self) -> Option<(ValueResourceKind, u32)> {
        self.is_resource().then(|| {
            let kind = ((self.0 & RESOURCE_KIND_MASK) >> RESOURCE_KIND_SHIFT) as u8;
            (
                ValueResourceKind::from_byte(kind),
                (self.0 & u32::MAX as u64) as u32,
            )
        })
    }
}
