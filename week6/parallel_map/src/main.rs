use crossbeam_channel;
use std::{thread, time};

// mod with_mutex;

// use with_mutex::parallel_map;

fn parallel_map<T, U, F>(mut input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default,
{
    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    output_vec.resize_with(input_vec.len(), Default::default);
    // TODO: implement parallel map!
    let (sender_in, reciever_in) = crossbeam_channel::unbounded();
    let (sender_out, reciever_out) = crossbeam_channel::unbounded();

    let mut threads = Vec::new();

    for _ in 0..num_threads {
        let reciever_in = reciever_in.clone();
        let sender_out = sender_out.clone();
        threads.push(thread::spawn(move || {
            while let Ok(input) = reciever_in.recv() {
                let (index, value) = input;
                sender_out.send( (index, f(value)) );
            } 
        }));
    }

    let length = input_vec.len();
    for i in 0..length {
        sender_in.send( ( length - i - 1, input_vec.pop().unwrap() ) ).expect("Tried writing to channel, but there are no receivers!");
    }

    drop(sender_in);
    drop(sender_out); 
    
    while let Ok(output) = reciever_out.recv() {
        let (index, value) = output;
        output_vec[index] = value;
    } 

    for thread in threads {
        thread.join().expect("Panic occurred in thread");
    }

    
    output_vec
}

fn main() {
    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("squares: {:?}", squares);
}
