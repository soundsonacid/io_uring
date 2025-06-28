#![allow(unused_unsafe, unused_variables, dead_code)]

use io_uring::*;
use libc::{
    MAP_POPULATE, MAP_SHARED, PROT_READ, PROT_WRITE, c_int, c_long, c_longlong, mmap, syscall,
};
use std::{mem::size_of, ops::Add};

// from /usr/include/x86_64-linux-gnu/asm/unistd_64.h
const IO_URING_SETUP: c_long = 425;
const IO_URING_ENTER: c_long = 426;
const IO_URING_REGISTER: c_long = 427;

const IORING_OFF_SQ_RING: c_longlong = 0;
const IORING_OFF_SQES: c_longlong = 0x10000000;

const IORING_OFF_CQ_RING: c_longlong = 0x8000000;

fn io_uring_setup(entries: u32, params: *mut io_uring_params) -> c_long {
    unsafe { syscall(IO_URING_SETUP, entries, params) }
}

fn io_uring_enter(
    fd: c_int,
    to_submit: c_int,
    min_complete: c_int,
    flags: c_int,
    sig: *mut libc::sigset_t,
) -> c_long {
    unsafe { syscall(IO_URING_ENTER, fd, to_submit, min_complete, flags, sig) }
}

fn read_barrier() {
    std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire);
}

fn write_barrier() {
    std::sync::atomic::fence(std::sync::atomic::Ordering::Release);
}

fn main() {
    let sq_off = io_sqring_offsets {
        head: 0,
        tail: 0,
        ring_mask: 0,
        ring_entries: 8,
        flags: 0,
        dropped: 0,
        array: 0,
        resv1: 0,
        user_addr: 0,
    };

    let cq_off = io_cqring_offsets {
        head: 0,
        tail: 0,
        ring_mask: 0,
        ring_entries: 8,
        overflow: 0,
        cqes: 0,
        resv1: 0,
        user_addr: 0,
    };

    let mut params = io_uring_params {
        sq_entries: 8,
        cq_entries: 8,
        flags: 0,
        sq_thread_cpu: 0,
        sq_thread_idle: 0,
        features: 0,
        wq_fd: 0,
        resv: [0; 3],
        sq_off,
        cq_off,
    };

    let ret = io_uring_setup(8, &mut params as *mut _);
    if ret < 0 {
        eprintln!("io_uring_setup failed with error code: {}", ret);
        return;
    }
    let ring_fd: c_int = ret as c_int;

    println!("io_uring_setup succeeded with return value: {}", ring_fd);
    println!("io_uring_params: {:?}", params);

    let sq_ptr = unsafe {
        mmap(
            std::ptr::null_mut(),
            // correctly use the params.sq_off instead of the original sq_off & the params.sq_entries instead of the sq_off.ring_entries
            params.sq_off.array as usize + params.sq_entries as usize * size_of::<u32>(),
            PROT_READ | PROT_WRITE,
            MAP_SHARED | MAP_POPULATE,
            ring_fd,
            IORING_OFF_SQ_RING,
        )
    };

    if sq_ptr == libc::MAP_FAILED {
        eprintln!("mmap failed with error code: {}", unsafe {
            *libc::__errno_location()
        });
        return;
    }

    println!("mmap succeeded, sq_ptr: {:?}", sq_ptr);

    let sq_entries = unsafe {
        mmap(
            std::ptr::null_mut(),
            // correctly use the params.sq_entries instead of the sq_off.ring_entries
            params.sq_entries as usize * size_of::<io_uring_sqe>(),
            PROT_READ | PROT_WRITE,
            MAP_SHARED | MAP_POPULATE,
            ring_fd,
            IORING_OFF_SQES,
        )
    };

    if sq_entries == libc::MAP_FAILED {
        eprintln!("mmap for sq_entries failed with error code: {}", unsafe {
            *libc::__errno_location()
        });
        return;
    }

    println!(
        "mmap for sq_entries succeeded, sq_entries: {:?}",
        sq_entries
    );

    let cq_ptr = unsafe {
        mmap(
            std::ptr::null_mut(),
            // correctly use the params.cq_off instead of the original cq_off & the params.cq_entries instead of the cq_off.ring_entries
            params.cq_off.cqes as usize + params.cq_entries as usize * size_of::<io_uring_cqe>(),
            PROT_READ | PROT_WRITE,
            MAP_SHARED | MAP_POPULATE,
            ring_fd,
            IORING_OFF_CQ_RING,
        )
    };

    if cq_ptr == libc::MAP_FAILED {
        eprintln!("mmap for cq_ptr failed with error code: {}", unsafe {
            *libc::__errno_location()
        });
        return;
    }

    println!("mmap for cq_ptr succeeded, cq_ptr: {:?}", cq_ptr);

    let cq_ptr_addr = cq_ptr as usize;
    let cq_h = std::thread::spawn(move || unsafe {
        println!("cqe thread started");
        let mut count = 1;
        // correctly use the params.cq_off instead of the original cq_off
        let cq_off = params.cq_off;
        loop {
            // locate the current head of the completion queue
            let head = cq_ptr_addr.add(cq_off.head as usize) as *mut u32;
            let head_val = *head;
            read_barrier();
            // check if the current head is equal to the tail
            // if it is not, then there is an event for us to read
            let tail = cq_ptr_addr.add(cq_off.tail as usize) as *mut u32;
            let tail_val = *tail;
            if head_val != tail_val {
                // get the current index of the completion queue
                let mask = *(cq_ptr_addr.add(cq_off.ring_mask as usize) as *mut u32);
                let index = head_val & mask;
                // get the array of completion queue entries (cast as io_uring_cqe for sizing / alignment at 16 bytes)
                let cqes = cq_ptr_addr.add(cq_off.cqes as usize) as *mut io_uring_cqe;
                // read the completion queue entry at the current index
                // we move by 16 bytes per index.
                // since size_of::<io_uring_cqe>() is 16 bytes, our pointer arithmetic 'strides' by 16 bytes per increment
                // without this, we would be reading the wrong / misaligned memory
                let cqe = cqes.add(index as usize) as *mut io_uring_cqe;
                // process the cqe
                println!("{count} completed cqe: {:?}", *cqe);
                count += 1;
                // increment the head
                *head = head_val + 1;
                write_barrier();
            }
        }
    });

    let sq_ptr_addr = sq_ptr as usize;
    let sq_entries_ptr = sq_entries as usize;
    let sq_h = std::thread::spawn(move || unsafe {
        println!("sqe thread started");
        let mut count = 1;
        // correctly use the params.sq_off instead of the original sq_off
        let sq_off = params.sq_off;
        loop {
            // get the current tail & associated value of the ring
            let tail = sq_ptr_addr.add(sq_off.tail as usize) as *mut u32;
            let tail_val = *tail;
            // get the mask of the sqe
            let mask = *(sq_ptr_addr.add(sq_off.ring_mask as usize) as *mut u32);
            // get the index of the sqe
            let index = tail_val & mask;
            // get the sqe at the current index from the sq entries
            let sqe_ptr = (sq_entries_ptr as *mut io_uring_sqe).add(index as usize);
            make_nop(sqe_ptr, count);
            // fill the sqe index into the sq ring array
            let array = sq_ptr_addr.add(sq_off.array as usize) as *mut u32;
            *array.add(index as usize) = index;
            /*
                correctly call io_uring_enter to submit the sqe
                we are not waiting for completions (since the cq thread is doing that), so min_complete = 0
                we are submitting 1 sqe at a time, so to_submit = 1
                no flags so flags = 0
                sig is null because we don't really care here

                otherwise, for sig:

                ret = io_uring_enter(fd, 0, 1, IORING_ENTER_GETEVENTS, &sig);

                is equivalent to atomically executing the following calls:

                pthread_sigmask(SIG_SETMASK, &sig, &orig);
                ret = io_uring_enter(fd, 0, 1, IORING_ENTER_GETEVENTS, NULL);
                pthread_sigmask(SIG_SETMASK, &orig, NULL);

            */
            let ret = io_uring_enter(
                ring_fd,
                1, // to_submit
                0, // min_complete
                0, // flags
                std::ptr::null_mut(), // sig
            );
            if ret < 0 {
                eprintln!("io_uring_enter failed with error code: {}", ret);
                return;
            }
            write_barrier();
            // increment the tail
            *tail = tail_val + 1;
            write_barrier();
            println!("{count} submitted sqe");
            count += 1;
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    cq_h.join().unwrap();
    sq_h.join().unwrap();
}

fn make_nop(sqe: *mut io_uring_sqe, count: u64) {
    unsafe {
        std::ptr::write_bytes(sqe, 0, 1);
        (*sqe).opcode = 0;
        (*sqe).fd = -1;
        (*sqe).u2.addr = 0;
        (*sqe).len = 0;
        (*sqe).u1.off = 0;
        (*sqe).user_data = count;
    }
}
