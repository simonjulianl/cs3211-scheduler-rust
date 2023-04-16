use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

pub type TaskResult = (
    u64,
    Option<Task>, // child
    Option<Task> // sibling
);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TaskType {
    Hash,
    Derive,
    Random,
}

static TYPE_ARRAY: [TaskType; 3] = [TaskType::Hash, TaskType::Derive, TaskType::Random];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub typ: TaskType,
    pub seed: u64,
    pub height: usize,
    pub max_children: usize,
    pub current_sibling_idx: usize,
    pub max_siblings: usize,
    pub rng: ChaCha20Rng, // use the rng first before you are passing it
}

fn generate_set(seed: u64, height: usize, max_children: usize, max_num: usize) -> Task {
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let num_tasks: usize = rng.gen_range(0..=max_num);
    Task {
        typ: TYPE_ARRAY[rng.gen_range(0..TYPE_ARRAY.len())],
        seed: rng.gen(),
        height,
        max_children,
        current_sibling_idx: 0,
        max_siblings: num_tasks,
        rng,
    }
}

impl Task {
    pub fn execute(&self) -> TaskResult {
        let output = match self.typ {
            TaskType::Hash => do_hash(self),
            TaskType::Derive => do_derive(self),
            TaskType::Random => do_random(self),
        };
        let (child_task, sibling_task) = self.get_next(output);
        (output, child_task, sibling_task)
    }

    fn get_next(&self, output: u64) -> (Option<Task>, Option<Task>) {
        // println!("Height: {} current idx {} max {}", self.height, self.current_sibling_idx, self.max_siblings);
        let task = if self.height <= 0 { None } else {
            let seed = self.seed ^ output;
            let mut rng = ChaCha20Rng::seed_from_u64(seed);
            let num_tasks: usize = rng.gen_range(0..=self.max_children);
            if num_tasks > 0 {
                Some(Task {
                    typ: TYPE_ARRAY[rng.gen_range(0..TYPE_ARRAY.len())],
                    seed: rng.gen(),
                    height: self.height - 1,
                    max_children: self.max_children,
                    current_sibling_idx: 0,
                    max_siblings: num_tasks,
                    rng,
                })
            } else { None }
        };

        return if self.current_sibling_idx + 1 >= self.max_siblings {
            (task, None)
        } else {
            let mut rng = self.rng.clone();
            (task, Some(Task {
                typ: TYPE_ARRAY[rng.gen_range(0..TYPE_ARRAY.len())],
                seed: rng.gen(),
                height: self.height,
                max_children: self.max_children,
                current_sibling_idx: self.current_sibling_idx + 1,
                max_siblings: self.max_siblings,
                rng,
            }))
        };
    }

    pub fn generate_initial(seed: u64, starting_height: usize, max_children: usize) -> Task {
        generate_set(seed, starting_height, max_children, 64)
    }
}

fn do_hash(task: &Task) -> u64 {
    let mut rng = ChaCha20Rng::seed_from_u64(task.seed);
    let rounds: usize = rng.gen_range(0x10000..0x20000);
    let mut state: [u8; 32] = [0; 32];
    rng.fill_bytes(&mut state);

    for _ in 0..rounds {
        let result = ring::digest::digest(&ring::digest::SHA256, &state);
        state.copy_from_slice(result.as_ref());
    }

    let take_from = rng.gen_range(0..(state.len() - std::mem::size_of::<u64>()));
    u64::from_le_bytes(state[take_from..take_from + 8].try_into().unwrap())
}

fn do_derive(task: &Task) -> u64 {
    let mut rng = ChaCha20Rng::seed_from_u64(task.seed);
    let mut state: [u8; 64] = [0; 64];
    let mut out: [u8; 64] = [0; 64];
    rng.fill_bytes(&mut state);
    ring::pbkdf2::derive(
        ring::pbkdf2::PBKDF2_HMAC_SHA512,
        rng.gen_range(0x10000..0x20000).try_into().unwrap(),
        &state[..32],
        &state[32..],
        &mut out[..],
    );

    let take_from = rng.gen_range(0..(out.len() - std::mem::size_of::<u64>()));
    u64::from_le_bytes(out[take_from..take_from + 8].try_into().unwrap())
}

fn do_random(task: &Task) -> u64 {
    let mut rng = ChaCha20Rng::seed_from_u64(task.seed);
    let rounds: usize = rng.gen_range(0x10000..0x20000);
    for _ in 0..rounds {
        rng.gen::<u64>();
    }
    rng.gen()
}
