use std::arch::asm;

pub fn assembly_dereference() -> usize {
    let t = 100;
    let t_ptr: *const usize = &t;
    // let t_ptr = 99999999999999 as *const usize;

    let x = dereference(t_ptr);

    x
}

fn dereference(ptr: *const usize) -> usize {
    let mut res: usize;
    unsafe {
        asm!("mov {0}, [{1}]", out(reg) res, in(reg) ptr);
    };
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assembly_dereference() {
        let result = assembly_dereference();
        assert_eq!(result, 100);
    }
}
