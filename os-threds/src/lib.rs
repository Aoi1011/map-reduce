use std::thread::sleep;

// Spawn several OS threds and put them on sleep.
// Sleeping is essentially the same as yielding to the OS scheduler with a request to be
// re-scheduled to run after a certain time has passed.
pub fn run_task() {
    println!("So, we start the program here.");
    let t1 = std::thread::spawn(move || {
        sleep(std::time::Duration::from_millis(200));
        println!("The long running tasks finish last!");
    });

    let t2 = std::thread::spawn(move || {
        sleep(std::time::Duration::from_millis(100));
        println!("We can chain callbacks");
        let t3 = std::thread::spawn(move || {
            sleep(std::time::Duration::from_millis(50));
            println!("...like this!");
        });
        t3.join().unwrap();
    }); 

    t1.join().unwrap();
    t2.join().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_task() {
        run_task();
    }
}
