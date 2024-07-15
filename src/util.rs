pub fn vec_into_sorted<T: Ord>(mut x: Vec<T>) -> Vec<T> {
    x.sort();
    x
}

