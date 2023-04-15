use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
    sync::atomic::{AtomicU64, Ordering},
    sync::{Mutex, Arc},
};

use task::{Task, TaskType};
use threadpool::ThreadPool;
use num_cpus;

fn main() {
    let (seed, starting_height, max_children) = get_args();

    eprintln!(
        "Using seed {}, starting height {}, max. children {}",
        seed, starting_height, max_children
    );

    let n_cpus = num_cpus::get();
    let pool = ThreadPool::new(n_cpus);

    let mut count_map = HashMap::new();
    let taskq = Arc::new(Mutex::new(VecDeque::from(Task::generate_initial(seed, starting_height, max_children))));

    static OUTPUT: AtomicU64 = AtomicU64::new(0);

    let start = Instant::now();
    while taskq.lock().unwrap().len() > 0 {
        let mut tq = taskq.lock().unwrap();
        while let Some(next) = tq.pop_front() {
            let taskq = taskq.clone();
            *count_map.entry(next.typ).or_insert(0usize) += 1;
            pool.execute(move || {
                let result = next.execute();
                OUTPUT.fetch_xor(result.0, Ordering::Relaxed);
                taskq.lock().unwrap().extend(result.1.into_iter());
            });
        }
        drop(tq); // if tq is not dropped, the mutex will be held by the main thread, preventing write from worker threads
        pool.join();
    }
    let end = Instant::now();

    eprintln!("Completed in {} s", (end - start).as_secs_f64());

    println!(
        "{},{},{},{}",
        OUTPUT.load(Ordering::Relaxed),
        count_map.get(&TaskType::Hash).unwrap_or(&0),
        count_map.get(&TaskType::Derive).unwrap_or(&0),
        count_map.get(&TaskType::Random).unwrap_or(&0)
    );
}

// There should be no need to modify anything below

fn get_args() -> (u64, usize, usize) {
    let mut args = std::env::args().skip(1);
    (
        args.next()
            .map(|a| a.parse().expect("invalid u64 for seed"))
            .unwrap_or_else(|| rand::Rng::gen(&mut rand::thread_rng())),
        args.next()
            .map(|a| a.parse().expect("invalid usize for starting_height"))
            .unwrap_or(5),
        args.next()
            .map(|a| a.parse().expect("invalid u64 for seed"))
            .unwrap_or(5),
    )
}

mod task;
