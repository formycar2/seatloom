// Need-to-know access enforcement: seats receive only their assigned scope.
// Full implementation deferred to data-engine packet.
#[derive(Default)]
pub struct IsolationLayer;

impl IsolationLayer {
    pub fn new() -> Self {
        Self
    }
}
