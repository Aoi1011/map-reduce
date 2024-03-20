`"mov rax, 1",`
Puts the value 1 in the `rax` register. When the CPU traps  our call later on a nd passes control to the OS, the kernel knows that a value of one in `rax` means that we want to make a `write`.

`"mov rdi, 1",`
Puts the value 1 in the `rdi` register. This tells the kernel where we want to write to, and a value of one means that we want to write to `stdout`.

`"syscall",`
`syscall` instruction. 
This instruction issues a software interrupt, and the CPU passes on control to the OS.

`in("rsi") msg_ptr,`
Writes the address to the buffer where our text is stored in the `rsi` register.

`in("rdx") len,`
Writes the length (in bytes) of our text buffer to the `rdx` register.

```
out("rax") _,
out("rdi") _,
lateout("rsi") _,
lateout("rdx") _
```
The next 4 lines are not instructions to the CPU; they're meant to tell the compiler that it can't store anything in these registers and assume the data is untouched when we exit the inline assembly block.


```
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
```
