pub fn is_all_same<T: PartialEq, U: Iterator<Item = T>>(mut el: U) -> bool {
    match el.next() {
        None => true,
        Some(e) => {
            for elem in el {
                if elem != e {
                    return false;
                }
            }

            true
        }
    }
}
