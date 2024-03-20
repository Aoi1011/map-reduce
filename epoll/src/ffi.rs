pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLLIN: i32 = 0x1;
pub const EPOLLET: i32 = 1 << 31;

#[link(name = "c")]
extern "C" {
    /// `epoll_create` is the syscall we make to create an epoll queue. [doc](https://man7.org/linux/man-pages/man2/epoll_create.2.html)
    /// The argument will be ignored but most have a value larger than 0.
    pub fn epoll_create(size: i32) -> i32;

    /// `close` is the syscall we need to close the file descriptor we get when we create our epoll
    /// instance
    pub fn close(fd: i32) -> i32;

    /// `epoll_ctl` is the control interface we use to perform operations on our epoll instance.
    /// `epfd` is the epoll file descriptor we want to perform operations on.
    /// `op` is the argument where we specify whether we want to perform an add, modify or delete
    /// operation
    pub fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *mut Event) -> i32;

    /// `epoll_wait` is the call that will block the current thred and wait until one of two things
    /// happens
    pub fn epoll_wait(epfd: i32, events: *mut Event, maxevents: i32, timeout: i32) -> i32;
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct Event {
    pub(crate) events: u32,
    pub(crate) epoll_data: usize,
}

impl Event {
    pub fn token(&self) -> usize {
        self.epoll_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(bitmask: i32) {
        const EPOLLONESHOT: i32 = 0x40000000;

        let read = bitmask & EPOLLIN != 0;
        assert!(read);

        let et = bitmask & EPOLLET != 0;
        assert!(et);

        let oneshot = bitmask & EPOLLONESHOT != 0;
        assert!(!oneshot);
    }

    #[test]
    fn test_bitflag() {
        let bitflag_a: i32 = 1 << 31;
        let bitflag_b: i32 = 0x1;
        let bitmask: i32 = bitflag_a | bitflag_b;

        check(bitmask);
    }
}
