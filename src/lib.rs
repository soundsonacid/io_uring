#![allow(non_camel_case_types)]
use libc::__kernel_rwf_t;

#[repr(C)]
#[derive(Debug)]
pub struct io_uring_params {
    pub sq_entries: u32,
    pub cq_entries: u32,
    pub flags: u32,
    pub sq_thread_cpu: u32,
    pub sq_thread_idle: u32,
    pub features: u32,
    pub wq_fd: u32,
    pub resv: [u32; 3],
    pub sq_off: io_sqring_offsets,
    pub cq_off: io_cqring_offsets,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct io_sqring_offsets {
    pub head: u32,
    pub tail: u32,
    pub ring_mask: u32,
    pub ring_entries: u32,
    pub flags: u32,
    pub dropped: u32,
    pub array: u32,
    pub resv1: u32,
    pub user_addr: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct io_cqring_offsets {
    pub head: u32,
    pub tail: u32,
    pub ring_mask: u32,
    pub ring_entries: u32,
    pub overflow: u32,
    pub cqes: u32,
    pub resv1: u32,
    pub user_addr: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct io_uring_cqe {
    pub user_data: u64,
    pub res: i32,
    pub flags: u32,
    pub big_cqe: [u64; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct io_uring_sqe {
    pub opcode: u8,
    pub flags: u8,
    pub ioprio: u16,
    pub fd: i32,
    pub u1: u1,
    pub u2: u2,
    pub len: u32,
    pub u3: u3,
    pub user_data: u64,
    pub u4: u4,
    pub personality: u16,
    pub u5: u5,
    pub u6: u6,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union u1 {
    pub off: u64,
    pub addr2: u64,
    pub u1_struct: u1_struct,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct u1_struct {
    pub cmp_op: u32,
    _pad1: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union u2 {
    pub addr: u64,
    pub splice_off_in: u64,
    pub u2_struct: u2_struct,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct u2_struct {
    pub level: u32,
    pub optname: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union u3 {
    pub rw_flags: __kernel_rwf_t,
    pub fsync_flags: u32,
    pub poll_events: u16,
    pub poll32_events: u32,
    pub sync_range_flags: u32,
    pub msg_flags: u32,
    pub timeout_flags: u32,
    pub accept_flags: u32,
    pub cancel_flags: u32,
    pub open_flags: u32,
    pub statx_flags: u32,
    pub fadvise_advice: u32,
    pub splice_flags: u32,
    pub rename_flags: u32,
    pub unlink_flags: u32,
    pub hardlink_flags: u32,
    pub xattr_flags: u32,
    pub msg_ring_flags: u32,
    pub uring_cmd_flags: u32,
    pub waitid_flags: u32,
    pub futex_flags: u32,
    pub install_fd_flags: u32,
    pub nop_flags: u32,
    pub pipe_flags: u32,
}

#[repr(packed)]
#[derive(Clone, Copy)]
pub union u4 {
    pub buf_index: u16,
    pub buf_group: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union u5 {
    pub splice_fd_in: i32,
    pub file_index: u32,
    pub zcrz_ifq_idx: u32,
    pub optlen: u32,
    pub u5_struct1: u5_struct1,
    pub u5_struct2: u5_struct2,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct u5_struct1 {
    pub addr_len: u16,
    pub _pad3: [u16; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct u5_struct2 {
    pub write_stream: u8,
    pub _pad4: [u8; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union u6 {
    pub u6_struct1: u6_struct1,
    pub u6_struct2: u6_struct2,
    pub optval: u64,
    pub cmd: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct u6_struct1 {
    pub addr3: u64,
    pub _pad2: [u64; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct u6_struct2 {
    pub attr_ptr: u64,
    pub attr_type_mask: u64,
}
