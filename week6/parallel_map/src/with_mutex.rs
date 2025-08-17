use std::sync::Arc;
use std::sync::Mutex;
use std::{thread, time};

pub fn parallel_map<T, U, F>(mut input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default + std::fmt::Debug,
{

    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    output_vec.resize_with(input_vec.len(), Default::default);
    // TODO: implement parallel map!

    let input_vec_reference = Arc::new(Mutex::new(input_vec));
    let output_vec_reference = Arc::new(Mutex::new(output_vec));

    let mut threads = Vec::new();

    for _ in 0..num_threads {
        let input_vec_reference = input_vec_reference.clone();
        let output_vec_reference = output_vec_reference.clone();
        threads.push(thread::spawn(move || {
            let mut idx : usize;
            let mut val;
            loop {

                {
                    let mut input = input_vec_reference.lock().unwrap();
                    let length = (*input).len();
                    if length == 0 {
                        break;
                    }
                    idx = length - 1;
                    val = (*input).pop().unwrap();
                }
                let out = f(val);
                {
                    let mut output = output_vec_reference.lock().unwrap();
                    output[idx] = out;
                }

            }
        }));
    }

    for thread in threads {
        thread.join().expect("Panic occurred in thread");
    }
    
    Arc::try_unwrap(output_vec_reference).unwrap().into_inner().unwrap()
    // Arc::into_inner(output_vec_reference).unwrap()
}