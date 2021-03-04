use std::sync::atomic::{AtomicUsize, Ordering};

static STRUCT_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub type StructId = usize;

/// For performance reasons, some structs need a unique id to be
/// compared efficiently with one another.
/// In order to generate it, we use an atomic static variable
/// that is incremented for each struct.
pub fn next_id() -> StructId {
  STRUCT_COUNTER.fetch_add(1, Ordering::SeqCst)
}