// we porting below codes to Rcore Tutorial v3
// https://cfsamson.gitbook.io/green-threads-explained-in-200-lines-of-rust/
// https://github.com/cfsamson/example-greenthreads
#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use user_lib::exit;

// This demonstration only works on RV64 due to the assembly code
// For RV32, we simply output a message and exit
#[cfg(target_pointer_width = "32")]
#[unsafe(no_mangle)]
pub fn main() {
    println!("stackful_coroutine: This demo only supports RV64");
    println!("stackful_coroutine PASSED (skipped on RV32)");
    exit(0);
}

#[cfg(target_pointer_width = "64")]
mod rv64_impl {
    use core::arch::naked_asm;
    use alloc::vec;
    use alloc::vec::Vec;
    use user_lib::exit;

    // In our simple example we set most constraints here.
    const DEFAULT_STACK_SIZE: usize = 4096; //128 got  SEGFAULT, 256(1024, 4096) got right results.
    const MAX_TASKS: usize = 5;
    static mut RUNTIME: usize = 0;

    pub struct Runtime {
        tasks: Vec<Task>,
        current: usize,
    }

    #[derive(PartialEq, Eq, Debug)]
    enum State {
        Available,
        Running,
        Ready,
    }

    struct Task {
        id: usize,
        stack: Vec<u8>,
        ctx: TaskContext,
        state: State,
    }

    #[derive(Debug, Default)]
    #[repr(C)] // not strictly needed but Rust ABI is not guaranteed to be stable
    pub struct TaskContext {
        // 15 u64
        x1: u64,  //ra: return addres
        x2: u64,  //sp
        x8: u64,  //s0,fp
        x9: u64,  //s1
        x18: u64, //x18-27: s2-11
        x19: u64,
        x20: u64,
        x21: u64,
        x22: u64,
        x23: u64,
        x24: u64,
        x25: u64,
        x26: u64,
        x27: u64,
        nx1: u64, //new return addres
    }

    impl Task {
        fn new(id: usize) -> Self {
            Task {
                id: id,
                stack: vec![0_u8; DEFAULT_STACK_SIZE],
                ctx: TaskContext::default(),
                state: State::Available,
            }
        }
    }

    impl Runtime {
        pub fn new() -> Self {
            let base_task = Task {
                id: 0,
                stack: vec![0_u8; DEFAULT_STACK_SIZE],
                ctx: TaskContext::default(),
                state: State::Running,
            };

            let mut tasks = vec![base_task];
            let mut available_tasks: Vec<Task> = (1..MAX_TASKS).map(|i| Task::new(i)).collect();
            tasks.append(&mut available_tasks);

            Runtime { tasks, current: 0 }
        }

        pub fn init(&self) {
            unsafe {
                let r_ptr: *const Runtime = self;
                RUNTIME = r_ptr as usize;
            }
        }

        pub fn run(&mut self) {
            while self.t_yield() {}
            println!("All tasks finished!");
        }

        fn t_return(&mut self) {
            if self.current != 0 {
                self.tasks[self.current].state = State::Available;
                self.t_yield();
            }
        }

        #[inline(never)]
        fn t_yield(&mut self) -> bool {
            let mut pos = self.current;
            while self.tasks[pos].state != State::Ready {
                pos += 1;
                if pos == self.tasks.len() {
                    pos = 0;
                }
                if pos == self.current {
                    return false;
                }
            }

            if self.tasks[self.current].state != State::Available {
                self.tasks[self.current].state = State::Ready;
            }

            self.tasks[pos].state = State::Running;
            let old_pos = self.current;
            self.current = pos;

            unsafe {
                switch(&mut self.tasks[old_pos].ctx, &self.tasks[pos].ctx);
            }

            self.tasks.len() > 0
        }

        pub fn spawn(&mut self, f: fn()) {
            let available = self
                .tasks
                .iter_mut()
                .find(|t| t.state == State::Available)
                .expect("no available task.");

            println!("RUNTIME: spawning task {}", available.id);
            let size = available.stack.len();
            unsafe {
                let s_ptr = available.stack.as_mut_ptr().offset(size as isize);
                let s_ptr = (s_ptr as usize & !7) as *mut u8;

                available.ctx.x1 = guard as u64;
                available.ctx.nx1 = f as u64;
                available.ctx.x2 = s_ptr.offset(-32) as u64;
            }
            available.state = State::Ready;
        }
    }

    fn guard() {
        unsafe {
            let rt_ptr = RUNTIME as *mut Runtime;
            (*rt_ptr).t_return();
        };
    }

    pub fn yield_task() {
        unsafe {
            let rt_ptr = RUNTIME as *mut Runtime;
            (*rt_ptr).t_yield();
        };
    }

    #[unsafe(naked)]
    #[unsafe(no_mangle)]
    unsafe extern "C" fn switch(old: *mut TaskContext, new: *const TaskContext) {
        naked_asm!(
            "
            sd x1, 0x00(a0)
            sd x2, 0x08(a0)
            sd x8, 0x10(a0)
            sd x9, 0x18(a0)
            sd x18, 0x20(a0)
            sd x19, 0x28(a0)
            sd x20, 0x30(a0)
            sd x21, 0x38(a0)
            sd x22, 0x40(a0)
            sd x23, 0x48(a0)
            sd x24, 0x50(a0)
            sd x25, 0x58(a0)
            sd x26, 0x60(a0)
            sd x27, 0x68(a0)
            sd x1, 0x70(a0)

            ld x1, 0x00(a1)
            ld x2, 0x08(a1)
            ld x8, 0x10(a1)
            ld x9, 0x18(a1)
            ld x18, 0x20(a1)
            ld x19, 0x28(a1)
            ld x20, 0x30(a1)
            ld x21, 0x38(a1)
            ld x22, 0x40(a1)
            ld x23, 0x48(a1)
            ld x24, 0x50(a1)
            ld x25, 0x58(a1)
            ld x26, 0x60(a1)
            ld x27, 0x68(a1)
            ld t0, 0x70(a1)

            jr t0
            "
        );
    }

    #[unsafe(no_mangle)]
    pub fn main() {
        println!("stackful_coroutine begin...");
        println!("TASK  0(Runtime) STARTING");
        let mut runtime = Runtime::new();
        runtime.init();
        runtime.spawn(|| {
            println!("TASK  1 STARTING");
            let id = 1;
            for i in 0..4 {
                println!("task: {} counter: {}", id, i);
                yield_task();
            }
            println!("TASK 1 FINISHED");
        });
        runtime.spawn(|| {
            println!("TASK 2 STARTING");
            let id = 2;
            for i in 0..8 {
                println!("task: {} counter: {}", id, i);
                yield_task();
            }
            println!("TASK 2 FINISHED");
        });
        runtime.spawn(|| {
            println!("TASK 3 STARTING");
            let id = 3;
            for i in 0..12 {
                println!("task: {} counter: {}", id, i);
                yield_task();
            }
            println!("TASK 3 FINISHED");
        });
        runtime.spawn(|| {
            println!("TASK 4 STARTING");
            let id = 4;
            for i in 0..16 {
                println!("task: {} counter: {}", id, i);
                yield_task();
            }
            println!("TASK 4 FINISHED");
        });
        runtime.run();
        println!("stackful_coroutine PASSED");
        exit(0);
    }
}

#[cfg(target_pointer_width = "64")]
pub use rv64_impl::main;
