// tsrs-only: the watch's post-cycle work (watchmem.rs `Freer::after_cycle`).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;

use tsrs_core::memsplit::{self, HeapStats};
use tsrs_execute::watchmem::Freer;

use crate::isolated;

static COLLECTS: AtomicUsize = AtomicUsize::new(0);

fn count_collect(_force: bool) {
    COLLECTS.fetch_add(1, Ordering::SeqCst);
}

// The free thread frees what the worker-pool threads allocated during the cycle, and the allocator keeps such a block
// in its page until the thread that allocated it collects. That holds on every platform, so after a cycle the
// allocator is collected on every worker-pool thread and on the free thread, whatever the target.
#[test]
fn after_cycle_collects_on_every_pool_thread_and_the_free_thread() {
    if !isolated::in_child(
        "watchmem_test::after_cycle_collects_on_every_pool_thread_and_the_free_thread",
    ) {
        return;
    }
    memsplit::set_heap_hooks(HeapStats::default, count_collect);
    let freer = Freer::new(false);
    freer.after_cycle();
    let (done, finished) = mpsc::channel();
    freer.run(move || done.send(()).unwrap());
    finished.recv().unwrap();
    assert_eq!(
        COLLECTS.load(Ordering::SeqCst),
        tsrs_compiler::worker_pool().current_num_threads() + 1
    );
}
