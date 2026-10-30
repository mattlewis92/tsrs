// An embedder that runs its own server registers a shutdown hook on tsrs's signal thread.

fn register_shutdown_hook() {
    tsrs_cli::signals::on_interrupt(|| {});
}

#[test]
fn on_interrupt_is_public() {
    // Not called: a registered hook is process-wide state.
    let _: fn() = register_shutdown_hook;
}
