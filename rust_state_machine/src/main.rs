use futures::executor::LocalPool;
use futures::task::LocalSpawnExt;
use state_machine::async_frame::PollingPool;

// Usage example
fn main() {
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    let started = std::rc::Rc::new(std::cell::RefCell::new(false));
    let exited = std::rc::Rc::new(std::cell::RefCell::new(false));
    let value = std::rc::Rc::new(std::cell::RefCell::new(false)); // Assume this could be set to true elsewhere
    let loop_count = std::rc::Rc::new(std::cell::RefCell::new(0));

    let started_clone = started.clone();
    let exited_clone = exited.clone();
    let value_clone = value.clone();
    let loop_count_clone = loop_count.clone();

    let mut children = PollingPool::default();
    let blocker = children.new_blocker();

    spawner
        .spawn_local(async move {
            *started_clone.borrow_mut() = true;
            while !*value_clone.borrow() {
                println!("before");
                blocker.yield_control().await;
                println!("after");

                *loop_count_clone.borrow_mut() += 1;
            }
            *exited_clone.borrow_mut() = true;
        })
        .unwrap();

    // Run until no more work can be done
    println!("First {:?}", loop_count.borrow());
    pool.run_until_stalled();
    children.wake_children();
    println!("Running the executor: {:?}", loop_count.borrow());
    pool.run_until_stalled();
    children.wake_children();

    println!("Loop ran {} times", loop_count.borrow());
}
