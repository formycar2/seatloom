use seatloom_core::objects::seat::SeatIdentity;
use seatloom_core::storage::seat_registry::SeatRegistry;

/// List all seats registered in the current project.
/// Bounded assumption: project root is `std::env::current_dir()`.
/// Sort: by `name` ascending.
pub fn list_seats() -> Vec<SeatIdentity> {
    let root = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return vec![],
    };
    let registry = SeatRegistry::new(&root);
    registry.list_seat_identities().unwrap_or_default()
}
