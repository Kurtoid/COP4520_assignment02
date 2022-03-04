use rand::prelude::*;
use std::sync::{atomic::AtomicBool, mpsc, Arc, Mutex};
fn main() {
    // atomic
    let repr_chosen = Arc::new(AtomicBool::new(false));
    let should_stop = Arc::new(AtomicBool::new(false));
    let cake_eaten = Arc::new(AtomicBool::new(false));
    // array of channels to wake up the threads
    let mut wake_up_threads: Vec<mpsc::Sender<()>> = Vec::new();
    // allows threads to signal the minotaur that they've left the maze
    let (left_maze_tx, left_maze_rx) = mpsc::channel();

    let mut num_threads = 8;
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
            println!("Usage: ./problem1 <num_threads>");
            std::process::exit(1);
        }
    }
    // make NUM_THREADS immutable
    let num_threads = num_threads;
    println!("Number of threads: {}", num_threads);

    let mut guests = Vec::new();
    for i in 0..num_threads {
        let repr_chosen = repr_chosen.clone();
        let thread_id = i;
        let (send_maze_tx, send_maze_rx) = mpsc::channel();
        wake_up_threads.push(send_maze_tx);
        let left_maze_tx = left_maze_tx.clone();
        let should_stop = should_stop.clone();
        let cake_eaten = cake_eaten.clone();
        guests.push(std::thread::spawn(move || {
            let mut have_i_eaten = false;
            let mut i_am_the_repr = false;
            let mut counter = 0;
            loop{
                send_maze_rx.recv().unwrap();
                if should_stop.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                println!("Guest {} is in the maze", thread_id);
                // if no representation has been chosen yet, elect ourself
                if !repr_chosen.load(std::sync::atomic::Ordering::Relaxed) {
                    repr_chosen.store(true, std::sync::atomic::Ordering::Relaxed);
                    i_am_the_repr = true;
                    println!(
                        "Guest {} is the first in the maze, and is the representative",
                        thread_id
                    );
                    // leave the cake for the next guest
                    // we're the first guest, and we count as one
                    counter += 1;
                } else if i_am_the_repr {
                    // has the cake been eaten?
                    if cake_eaten.load(std::sync::atomic::Ordering::Relaxed) {
                        println!(
                            "Guest {} found the cake eaten, and sets the counter to {}",
                            thread_id, counter
                        );
                        // increment the counter
                        counter += 1;
                        if counter == num_threads {
                            // everyone has been to the maze
                            // hopefully, the last guest can request some cake for themselves
                            println!("Everyone has been to the maze");
                            should_stop.store(true, std::sync::atomic::Ordering::Relaxed);
                        } else {
                            // put the cake back
                            cake_eaten.store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                    } else {
                        // do nothing
                    }
                } else {
                    // has the cake been eaten?
                    if cake_eaten.load(std::sync::atomic::Ordering::Relaxed) {
                        // do nothing
                        println!("Guest {} does nothing, since there is no cake", thread_id);
                    } else if !have_i_eaten {
                        // eat the cake
                        println!("Guest {} eats the cake", thread_id);
                        cake_eaten.store(true, std::sync::atomic::Ordering::Relaxed);
                        have_i_eaten = true;
                    } else {
                        // do nothing
                        println!(
                            "Guest {} does nothing, since they have already eaten",
                            thread_id
                        );
                    }
                }
                // tell the minotaur that we've left the maze
                left_maze_tx.send(()).unwrap();
            }
        }));
    }
    // send one guest at a time, at random
    while !should_stop.load(std::sync::atomic::Ordering::Relaxed) {
        let guest_to_send = rand::thread_rng().gen_range(0..num_threads);
        wake_up_threads[guest_to_send].send(()).unwrap();
        left_maze_rx.recv().unwrap();
    }
    // tell the guests to stop
    // since the first guest set should_stop to true,
    // we just have to wake up the threads so they can stop
    for i in 0..num_threads {
        wake_up_threads[i].send(()).unwrap();
    }
    // wait for the guests to stop
    for guest in guests {
        guest.join().unwrap();
    }
}
