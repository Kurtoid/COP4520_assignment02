/*
 * Strategy 1 could work, but a failed attempt to enter the room while it's occupied
 * could be (time) costly. Strategy 2 works as an extension to stratey 1:
 * Strategy 2 makes the most sense. The sign, if considered as an
 * atomic boolean, would allow any guest/thread to check it's state
 * with very little cost/overhead. The queue in Strategy 3 means that 
 * waiting threads either won't be able to accomplish other tasks
 * while waiting, or that the room could spend time empty while the next
 * queued task is ready to enter.
 * 
 * Here, I implement strategy 2
 */
use rand::prelude::*;
use std::{sync::{atomic::AtomicBool, Arc}};
fn main() {
    let visitor_at_vase = Arc::new(AtomicBool::new(false));
    
    let mut num_threads = 8;
    let visitor_visit_count = 3;
    let visitor_visit_time = 0.1;
    let visitor_roll_time = 0.1;
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        2 => {
            num_threads = match args[1].parse::<usize>() {
                Ok(n) => n,
                Err(_) => {
                    println!("Invalid number of threads");
                    std::process::exit(1);
                }
            };
        }
        1 => {}
        _ => {
            println!("Usage: ./problem2 <num_threads>");
            std::process::exit(1);
        }
    }
    // make NUM_THREADS immutable
    let num_threads = num_threads;
    println!("Number of threads: {}", num_threads);

    let mut guests = Vec::new();
    for i in 0..num_threads {
        let visitor_at_vase = visitor_at_vase.clone();

        guests.push(std::thread::spawn(move || {
            let mut times_visited = 0;
            while times_visited < visitor_visit_count {
                // sleep for visitor_roll_time
                std::thread::sleep(std::time::Duration::from_millis(visitor_roll_time as u64));
                // 50% chance to try to visit the vase
                if rand::thread_rng().gen_range(0 .. 2) == 0 {
                    // if the vase isn't visited yet, try to visit it
                    if !visitor_at_vase.swap(true, std::sync::atomic::Ordering::Relaxed) {
                        // sleep for visitor_visit_time
                        std::thread::sleep(std::time::Duration::from_millis(visitor_visit_time as u64));
                        times_visited += 1;
                        // set the vase back to unvisited
                        visitor_at_vase.store(false, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        }));
    }

    for guest in guests {
        guest.join().unwrap();
    }
}
