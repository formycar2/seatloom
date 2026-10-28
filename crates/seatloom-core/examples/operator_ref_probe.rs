//! Print how the operator identity resolves on THIS machine, and how the refusal
//! reads. Used by proof/operator_ref_config_proof.sh so the acceptance criteria
//! are measured against the real file rather than asserted in prose.
//!
//!   cargo run -p seatloom-core --example operator_ref_probe

fn main() {
    match seatloom_core::config::read_operator_ref() {
        Ok(r) => {
            println!("RESOLVED value={} source={:?}", r.value, r.source);
        }
        Err(e) => {
            println!("REFUSED error={e}");
            println!("--- message as the operator sees it ---");
            println!(
                "{}",
                seatloom_core::config::operator_ref_refusal_message(&e)
            );
        }
    }
}
