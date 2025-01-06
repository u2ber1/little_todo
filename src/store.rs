use std::fs::{File, OpenOptions};
use std::path::Path;
use std::sync::mpsc::{Sender, Receiver};
use std::io::{
    prelude::*,
    BufReader, 
    BufRead, 
    BufWriter,
    SeekFrom,
    Seek
};

use crate::sdlr::TaskCommand;

pub struct Store {
    file: Option<File>
}

pub enum StoreToken {
    Store(Vec<String>),
    Read(Sender<TaskCommand>),
    Exit
}

impl Store {
    pub fn new(path: &str) -> Self {
        let pt = Path::new(path);
        let f = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(pt) {
                Ok(f) => Some(f),
                Err(_) => None
            };
        Store {
            file: f
        }
    }
    fn read_to_vec(&mut self) -> Result<Vec<String>, &str> {
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
    fn vec_to_file(&mut self, json: Vec<String>) -> bool {
        if let Some(ref mut file) = self.file {
            file.seek(SeekFrom::Start(0)).unwrap();
            file.set_len(0).unwrap();
            let mut writer = BufWriter::new(file);
            for item in json {
                println!("{}", item);
                if let Err(_) = writeln!(writer, "{}", item) {
                    return false;
                }
                writer.flush().unwrap();
            }
            true
        } else {
            false
        }
    }
    // 使用channel来完成
    // 这种交互方式好吗？
    pub fn monitor_store(mut self, rx: &Receiver<StoreToken>) {
        loop {
            while let Ok(i) = rx.recv() {
                match i {
                    StoreToken::Store(json) => {
                        self.vec_to_file(json);
                    }
                    StoreToken::Read(tx) => {
                        // println!("{:?}", self.read_to_vec());
                        match self.read_to_vec() {
                            Ok(json) => {
                                tx.send(
                                    TaskCommand::FromJson(json)
                                ).unwrap();
                            },
                            Err(e) => {
                                println!("{e}");
                                continue;
                            }
                        };
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