//! Execute the unchanged external-style quantity/scalar consumer.

#[path = "../../examples/provider_boundary.rs"]
mod consumer;

#[test]
fn public_provider_types_preserve_computed_values() {
    consumer::main();
}
