use std::{
    collections::HashMap,
    time::Instant,
    sync::mpsc::channel,
};

use task::{Task, TaskType};
use threadpool::ThreadPool;
use num_cpus;

// this would use 2563 MB in my local machine
fn main() {
    let (seed, starting_height, max_children) = get_args();
    let n_cpus = num_cpus::get();

    eprintln!(
        "Using seed {}, starting height {}, max. children {}, n cpus: {}",
        seed, starting_height, max_children, n_cpus
    );

    let mut count_map = HashMap::new();
    let init_task = Task::generate_initial(seed, starting_height, max_children);

    let mut task_counter: u128 = 1;

    let (send_ch, recv_ch) = channel();
    let th_pool = ThreadPool::new(n_cpus);

    let mut output: u64 = 0;

    let start = Instant::now();

    let typ = init_task.typ;
    *count_map.entry(typ).or_insert(0usize) += 1;
    let s = send_ch.clone();
    th_pool.execute(move || {
        s.send(init_task.execute())
            .expect("Please receive this task");
    });

    while task_counter > 0 {
        /*
         In every step do the following:
         1. Wait for result from task pool
         2. If there is more task, send it back to the pool

         Repeat this until all the tasks are finished
         */

        let (result, child_task, sibling_task) = recv_ch.recv().unwrap();
        match child_task {
            Some(new_task) => {
                task_counter += 1;
                let typ = new_task.typ;
                *count_map.entry(typ).or_insert(0usize) += 1;
                let send_ch = send_ch.clone();
                th_pool.execute(move || {
                    send_ch.send(new_task.execute())
                        .expect("Please receive this task");
                });
            },
            None => {}
        }

        match sibling_task {
            Some(new_task) => {
                task_counter += 1;
                let typ = new_task.typ;
                *count_map.entry(typ).or_insert(0usize) += 1;
                let send_ch = send_ch.clone();
                th_pool.execute(move || {
                    send_ch.send(new_task.execute())
                        .expect("Please receive this task");
                });
            },
            None => {}
        }

        output ^= result;
        task_counter -= 1;
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
