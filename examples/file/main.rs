use std::fs::{File, OpenOptions, metadata};
use std::io::{Write, BufRead, BufReader, Seek, Read};
use tokio::io::SeekFrom;

fn main() {
    // let mut file = OpenOptions::new()
    //     .append(true)
    //     .create(true)
    //     .open("profit.log")
    //     .expect("打开文件失败");
    // let str = format!("{} {}", chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"), "我日了鬼了。\n");
    // file.write(str.as_bytes()).unwrap();
    let mut file = File::open("output.log").unwrap();
    // let metadata = file.metadata().unwrap();
    let mut str = String::new();
    // file.seek(SeekFrom::Start(metadata.len() - 4096 * 4)).unwrap();
    file.read_to_string(&mut str);
    let mut log = String::new();
    let mut index = 0;
    for line in str.lines().rev() {
        log = format!("{}\n{}", log, line);
        if index > 30 { break }
        index += 1;
    }
    println!("{}", log);
}