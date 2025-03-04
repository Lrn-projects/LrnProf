# LrnProf

## Current state and context

LrnProf is a profiler fully written in Rust that currently targets only Mach-O binary files. It is written from scratch using hand parsing with structures obtained from Apple documentation.

I use several repositories explaining how a binary file is structured and the different tables in it. I also use Chat-GPT to explain many concepts that I didn't understand when I started this project.

Currently, the profiler is not fully functional. While some features are operational, the main goal of tracing the call stack and unwinding it for debugging purposes is not yet working. At first, I was parsing the binary file directly. From the PID, I was able to locate and parse the binary file directly to retrieve data inside it. But you cannot correctly retrace the call stack using this method, and that's not even the purpose of a profiler. So I restarted almost from the beginning.

Currently, the profiler can parse a target process from its PID and parse it directly from its address space. It can parse the Mach-O header, define what type of Mach-O it is (currently only 32 and 64 bits) using the magic number in little-endian.

After that, it can parse the load commands and store them in a vector. But I can only get 17 out of 18 (from ncmds) from the target process, and I don't know why. Then I can retrieve the symtab and some lc_segment_64, but they are almost all misaligned.

## Important concepts

The profiler can currently only parse mach-o binary, so binary file from MacOS. There's multiple binary format targeting specific arch or OS. Like ELF for linux.

When I started the project I made some research to know how I can do it. I first thought about syscalls but I found a crate called mach2 using some method to retrieve data from kernel. After some more research I find out that Apple is using an API called mach. It's like an "overlay" or a shell not like bash or zsh. It allow to retrieve data directly from the kernel, the Mach API expose multiple features without having to use syscalls. Like threads information, memory management, process management etc.

First thing I've done is to use the `task_for_pid` method. It allow to get a task port from another process, allowing the profiler to access the memory address space of the target process, and to read it or write to it. Getting information like threads, user or system time. To use it you need to pass the pid of the target process and a mutable pointer to retrieve the task port:

```rust
let mut task: u32 = 0;
let task_pid = task_for_pid(mach_task_self(), *pid, &mut task);
```

mach_task_self() is a method that return a task port for the current process(so the profiler). The goal to use it here is to attach the target process to the profiler if I understand it correctly. Next is to pass a pointer to pid and a mutable reference to a mutable u32. So with that you can use the task variable to retrieve information from the target process. task_for_pid is used to retrieve a task port from a given process and interact with it's address space. Why ? Because you cannot access an address space different that the current one. Like if you don't have a task port the profiler cannot retrieve any information from the given process. Even if you have the correct virtual memory address I guess. Because each process have there own address space.

After that I use the `task_threads` function. It is used to retrieve all threads information from the task port. I don't remember why I go to this function except to have some metrics about the process. Or I needed it to retrieve the call stack of the main thread of the process if I remember correctly. But that's not really necessary for the profiler to work.

Then I go for the `thread_info`. Like `task_threads` that's not necessary to make a profiler work but I really found the concept of flavor interesting. To use the `thread_info` method:

```rust
let mut thread_list: *mut u32 = std::ptr::null_mut();
let thread_basic_info: u32 = 3;
let mut thread_info_out: [i32; 1024] = [0; 1024];
let mut thread_info_out_cnt: u32 = 1024;

let thread_info = libc::thread_info(
    *thread_list,
    thread_basic_info,
    thread_info_out.as_mut_ptr(),
    &mut thread_info_out_cnt,
);
```

`thread_list` variable is just a pointer, the flavor a u32 and the rest is buffer. But if you pass a different flavor you get new information like totally different. Or almost, like it's used like a "filter ?" What do you want to retrieve from this function you pass the correct flavor and use the buffer or the pointer. And that's so cool. No use of return value from the function. Directly write to a pointer or mutable buffer and you can use them. But once more it's not useful to make a profiler work.

Next I use the `thread_get_state` function, it is used to get the state of all given thread, this function is used to get things like CPU registry, thread state(if is paused, locked, pending...). `thread_get_state` can be used to retrieve where each thread execute there code. You use it like `thread_info`, you passe the thread you want to analyze, a flavor, a mutable buffer and the buffer size. Then you use the data store in the buffer.

The next thing I do is to retrace the call stack of the main thread. I didn't have any use of it for know so I'll explain it anyway. When using the buffer fill from `thread_get_state`, we can retrace the call stack. We can retrace the call stack by following the pointers of the stack frame. The stack frame is how certain arch store data from the call stack. There:

- `FP`: `Frame Pointer` It point to the beginning of the next stack frame.
- `LR`: `Link Register` contain the return address, corresponding to the calling function
- `SP`: `Stack Pointer` point to the top of the stack
- `PC`: `Program counter` address of the current execute function

To retrace the call stack we need to follow the FP of each stack frame to retrieve previous FP.

We read FP to get:

- the `Next FP`
- the `Next LR`

We store `Next LR` and we repeat over until `Next FP == 0`. So we will be at the top of the stack.

```rust
let mut FP = new_state[29];
// link register (return addr)
let LR = new_state[30];
// stack pointer
let SP = new_state[31];
// program pointer
let PC = new_state[32];

if FP == 0 {
    panic!("FP is 0 cannot unreferenced");
}

let fp_ptr = FP as *const u64;

let pid_i32 = *pid as i32;
if !fp_ptr.is_null() {
    loop {
        let read_process_address = read_process_address(
            pid_i32,
            FP as usize,
            size,
            &mut process_address_bytes_buffer,
        )
        .unwrap();

        // load the specific chunk of the buffer into vector
        // used to get the next_fp and next_lr addresses
        next_fp_bytes_vec.copy_from_slice(&process_address_bytes_buffer[..8]);
        next_lr_bytes_vec.copy_from_slice(&process_address_bytes_buffer[8..16]);
        let next_fp = u64::from_le_bytes(next_fp_bytes_vec);
        let next_lr = u64::from_le_bytes(next_lr_bytes_vec);
        addresses.push(next_lr);
        let current_fp = FP;
        FP = next_fp;
        // println!("Next FP: {:#x}, Next LR: {:#x}", current_fp, next_lr);
        if next_fp == 0 {
            break;
        }
    }
}
```

