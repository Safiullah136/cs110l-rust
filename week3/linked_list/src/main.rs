use linked_list::LinkedList;
pub mod linked_list;

fn main() {
    let mut list: LinkedList<String> = LinkedList::new();
    assert!(list.is_empty());
    assert_eq!(list.get_size(), 0);
    for i in 1..12 {
        list.push_front(format!("Hi from {}\n", i));
    }
    println!("{}", list);
    println!("list size: {}", list.get_size());

    let mut list2 = list.clone();
    println!("Eq: {}", list == list2);

    println!("top element: {}", list.pop_front().unwrap());
    println!("{}", list);
    println!("size: {}", list.get_size());
    // println!("{}", list.to_string()); // ToString impl for anything impl Display

    println!("{}", list2);
    println!("Eq: {}", list == list2);

    // If you implement iterator trait:
    for val in &list {
       println!("{}", val);
    }

    for val in list {
        print!("{}", val);
    }
    // println!("original list : {}", list); // this line should cause compile err
}
