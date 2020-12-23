use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("B", "你好");
    map.insert("G", "哈哈");
    map.insert("C", "哈哈");
    map.insert("c", "哈哈");
    map.insert("z", "哈哈");
    map.
    for key in map.keys() {
        println!("{}", key);
    }
}