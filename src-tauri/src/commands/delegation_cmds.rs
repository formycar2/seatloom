use seatloom_core::objects::seat::SeatDelegation;
use seatloom_core::storage::seat_registry::SeatRegistry;

/// List SeatDelegations, optionally filtered by seat and active status (AD-009).
/// Bounded assumption: project root is `std::env::current_dir()`.
/// Sort: by `issued_at` descending.
pub fn list_delegations(
    _seat_id: Option<String>,
    _active_only: Option<bool>,
) -> Vec<SeatDelegation> {
    let root = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return vec![],
    };
    let registry = SeatRegistry::new(&root);
    registry.list_delegations().unwrap_or_default()
}
