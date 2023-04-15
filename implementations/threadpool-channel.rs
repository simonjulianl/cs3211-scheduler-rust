use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
    sync::mpsc::channel,
    sync::mpsc::Sender,
    sync::mpsc::Receiver,
};

use task::{Task, TaskType};
use threadpool::ThreadPool;
use num_cpus;

fn execute_task(send: &Sender<(u64, Vec<Task>)>, count_map: &mut HashMap<TaskType, usize>, spawned: &mut u64, pool: &ThreadPool, next: Task) {
    let send = send.clone();
    *count_map.entry(next.typ).or_insert(0usize) += 1;
    *spawned += 1;
    pool.execute(move || {
        send.send(next.execute()).unwrap();
    });
}

fn wait_task(recv: &Receiver<(u64, Vec<Task>)>, spawned: &mut u64, output: &mut u64) -> std::vec::IntoIter<Task> {
    let result = recv.recv().unwrap();
    *spawned -= 1;
    *output ^= result.0;
    result.1.into_iter()
}

fn main() {
    let (seed, starting_height, max_children) = get_args();

    eprintln!(
        "Using seed {}, starting height {}, max. children {}",
        seed, starting_height, max_children
    );

    let n_cpus = num_cpus::get();
    let pool = ThreadPool::new(n_cpus);

    let (send, recv) = channel();

    let mut count_map = HashMap::new();
    let mut taskq = VecDeque::from(Task::generate_initial(seed, starting_height, max_children));

    let mut output: u64 = 0;
    let mut spawned: u64 = 0;

    let start = Instant::now();

    while let Some(next) = taskq.pop_front() {
        execute_task(&send, &mut count_map, &mut spawned, &pool, next);
    }

    while spawned > 0 {
        let new_tasks = wait_task(&recv, &mut spawned, &mut output);
        for next in new_tasks {
            execute_task(&send, &mut count_map, &mut spawned, &pool, next);
        }
    }

    let end = Instant::now();

    eprintln!("Completed in {} s", (end - start).as_secs_f64());

    println!(
        "{},{},{},{}",
        output,
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
