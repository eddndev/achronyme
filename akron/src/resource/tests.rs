use super::*;

#[test]
fn ownership_transfer_and_exclusive_borrow_are_enforced() {
    let mut table = ResourceTable::default();
    let handle = table.reserve(ValueResourceKind::Connection, None).unwrap();
    let value = table
        .activate_network(handle, ValueResourceKind::Connection)
        .unwrap();

    table.transfer(handle, None, Some(7)).unwrap();
    assert!(table
        .require(value, ValueResourceKind::Connection, None)
        .is_err());
    table
        .borrow(handle, ValueResourceKind::Connection, Some(7), 8)
        .unwrap();
    assert!(table.transfer(handle, Some(7), Some(9)).is_err());
    table.release_borrow(handle, 8);
    table.transfer(handle, Some(7), Some(9)).unwrap();
}

#[test]
fn closed_handles_are_never_reused() {
    let mut table = ResourceTable::default();
    let first = table.reserve(ValueResourceKind::Channel, None).unwrap();
    assert_eq!(table.close(first), Some(ValueResourceKind::Channel));
    assert_eq!(table.close(first), None);
    let second = table.reserve(ValueResourceKind::Channel, None).unwrap();
    assert_ne!(second, first);
    assert!(table.entry(first).is_err());
}
