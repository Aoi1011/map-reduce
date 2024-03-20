use std::{arch::asm, io};

#[inline(never)]
pub fn syscall(message: String) {
    let msg_ptr = message.as_ptr();
    let len = message.len();

    unsafe {
        asm!(
            "mov rax, 1",
            "mov rdi, 1",
            "syscall",
            in("rsi") msg_ptr,
            in("rdx") len,
            out("rax") _,
            out("rdi") _,
            lateout("rsi") _,
            lateout("rdx") _
        );
    }
}

// Every Linux installation comes with a version of libc, which is a C library for communicating
// with the operating system.
#[link(name = "c")]
extern "C" {
    fn write(fd: u32, buf: *const u8, count: usize) -> i32;
}

/// `1` is always the file handle to *stdout* on UNIX systems.
pub fn normal_syscall(message: String) -> io::Result<()> {
    let msg_ptr = message.as_ptr();
    let len = message.len();
    let res = unsafe { write(1, msg_ptr, len) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall() {
        let message = "Hello world from raw syscall!\n";
        let message = String::from(message);
        syscall(message);
    }

    #[test]
    fn test_normal_syscall() {
        let message = "Hello world from raw normal syscall!\n";
        let message = String::from(message);
        normal_syscall(message).unwrap();
    }
}
