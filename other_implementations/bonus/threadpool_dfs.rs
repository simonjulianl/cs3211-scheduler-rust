use std::{
    collections::{HashMap, VecDeque},
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
    let mut taskq = VecDeque::new();
    let init_task = Task::generate_initial(seed, starting_height, max_children);
    taskq.push_back(init_task);

    let mut task_counter: u128 = 1;

    let (send_ch, recv_ch) = channel();
    let th_pool = ThreadPool::new(n_cpus);

    let mut output: u64 = 0;

    let start = Instant::now();
    while task_counter > 0 {
        /*
         In every step do the following:
         1. get the next task if any
         2. If there is, send is to task pool
         3. Attempt to recv any result

         Repeat this until all the tasks are finished
         */
        if let Some(next) = taskq.pop_front() {
            let typ = next.typ;
            *count_map.entry(typ).or_insert(0usize) += 1;
            let send_ch = send_ch.clone();
            th_pool.execute(move || {
                send_ch.send(next.execute())
                    .expect("Please receive this task");
            })
        }

        let task_result = recv_ch.try_recv();
        if task_result.is_ok() {
            let (result, child_task, sibling_task) = task_result.unwrap();
            if sibling_task.is_some() {
                taskq.push_front(sibling_task.unwrap());
                task_counter += 1;
            }

            if child_task.is_some() {
                taskq.push_front(child_task.unwrap());
                task_counter += 1;
            }

            output ^= result;
            task_counter -= 1;
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
