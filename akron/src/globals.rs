use memory::Value;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct GlobalEntry {
    pub value: Value,
    pub mutable: bool,
    pub defined: bool,
}

impl GlobalEntry {
    pub fn new(value: Value, mutable: bool) -> Self {
        Self {
            value,
            mutable,
            defined: true,
        }
    }

    pub(crate) fn undefined() -> Self {
        Self {
            value: Value::nil(),
            mutable: false,
            defined: false,
        }
    }
}

const _: () = assert!(std::mem::size_of::<GlobalEntry>() == 16);
const _: () = assert!(std::mem::offset_of!(GlobalEntry, value) == 0);
const _: () = assert!(std::mem::offset_of!(GlobalEntry, mutable) == 8);
const _: () = assert!(std::mem::offset_of!(GlobalEntry, defined) == 9);
