use std::fs::File;
use std::io::{prelude::*,BufReader, BufRead, BufWriter};
use std::path::Path;
use std::sync::mpsc::Receiver;

use crate::sdlr::TaskCommand;

pub struct Store {
    file: Option<File>
}

enum StoreToken {
    Store(Vec<String>),
    Read,
    Exit
}

impl Store {
    fn new(path: &str) -> Self {
        let pt = Path::new(path);
        let f = match File::open(&pt){
            Ok(f) => Some(f),
            Err(_) => {
                match File::create(&pt) {
                    Ok(f) => Some(f),
                    Err(_) => None
                }
            },
        };
        Store {
            file: f
        }
    }
    fn read_to_vec(&self) -> Result<Vec<String>, &str> {
        if self.file.is_none() {
            return Err("文件为空，无法读取");
        }
        let mut json: Vec<String> = Vec::new();
        let fp = self.file.as_ref().ok_or("文件引用失败")?;
        let reader = BufReader::new(fp);

        for i in reader.lines() {
            let li = match i {
                Ok(s) => s,
                Err(_) => continue
            };
            json.push(li);
        }
        Ok(json)
    }
    fn vec_to_file(&self, json: &Vec<String>) -> bool {
        if let Some(ref file) = self.file {
            let mut writer = BufWriter::new(file);
            for item in json {
                if let Err(_) = writeln!(writer, "{}", item) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
    // 使用channel来完成
    // 这种交互方式好吗？
    pub fn monitor_store(&self, tx: &Sender<TaskCommand>, rx: &Receiver<StoreToken>) {
        loop {
            while let Ok(i) = rx.recv() {
                match i {
                    StoreToken::Store(json) => {
                        self.vec_to_file(&json);
                    }
                    StoreToken::Read() => {
                        println!("{:?}", self.read_to_vec());
                    }
                    StoreToken::Exit => {
                        std::process::exit(-1);
                    }
                }
            }
        }
    }
}

#[cfg(test)] 
mod test {
    #[test]
    fn file_to_json() {
        println!("hello");
    }
    #[test]
    fn json_to_file() {
        assert_eq!(1, 1);
    }
}