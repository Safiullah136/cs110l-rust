/* The following exercises were borrowed from Will Crichton's CS 242 Rust lab. */

use std::collections::HashSet;

fn main() {
    let mut v : Vec<i32> = Vec::new();
    v.push(1);
    v.push(2);
    v.push(2);    
    dedup(&mut v);
    println!("Hi! Try running \"cargo test\" to run tests.");
}

fn add_n(v: Vec<i32>, n: i32) -> Vec<i32> {
    let mut v_clone = v.clone();
    for i in v_clone.iter_mut() {
        *i = *i + n;
    }
    v_clone
}

fn add_n_inplace(v: &mut Vec<i32>, n: i32) {
    for i in v.iter_mut() {
        *i = *i + n;
    }
}

fn dedup(v: &mut Vec<i32>) {
    let mut dict = HashSet::new();
    let mut v_new = Vec::new();
    for i in v.iter() {
        if dict.contains(i) {
            continue;
        } else {
            dict.insert(*i);
            v_new.push(*i);
            println!("{}", *i);
        }
    }
    v.clear();
    for i in v_new.iter() {
        v.push(*i);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_n() {
        assert_eq!(add_n(vec![1], 2), vec![3]);
    }

    #[test]
    fn test_add_n_inplace() {
        let mut v = vec![1];
        add_n_inplace(&mut v, 2);
        assert_eq!(v, vec![3]);
    }

    #[test]
    fn test_dedup() {
        let mut v = vec![3, 1, 0, 1, 4, 4];
        dedup(&mut v);
        assert_eq!(v, vec![3, 1, 0, 4]);
    }
}
