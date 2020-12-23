use std::fs::{File, OpenOptions};
use std::io::Write;

fn main() {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("profit.log")
        .expect("打开文件失败");
    let str = format!("{} {}", chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"), "我日了鬼了。\n");
    file.write(str.as_bytes()).unwrap();
}