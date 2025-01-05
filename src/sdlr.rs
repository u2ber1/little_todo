use std::sync::mpsc::Receiver;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use std::fmt;
use std::collections::HashMap;
use std::fmt::{Display};
use serde_json::to_string;
use serde::{Deserialize, Serialize};
use crate::store::StoreToken;

#[derive(Serialize, Deserialize)]
pub struct Scheduler {
	name: String,
	msg: String,
	stime: u64,
	prio: u16,
}

type ID = u64;
pub enum TaskCommand {
	Add(String, String, u64, u16),
	Remove(ID),
	Update(ID, Option<String>, Option<String>, Option<u64>, Option<u16>),
	List,
	CheckTime(u64),
	Json/*用管道来完成*/,
	Exit,
}

pub struct SchedulerList {
	sl: Arc<Mutex<HashMap<ID, Scheduler>>>,
	uid: Arc<Mutex<ID>>,
}

impl Display for Scheduler {

fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
write!(f,
"<name>: {}
	[msg]: {}	
	[pri]:{}
	[time]: {}
", self.name, self.msg, self.prio, self.stime)
	}
}

fn now() -> u64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs()
}

impl Default for Scheduler {
	fn default() -> Self {
		Self {
			name: "d_name".to_string(),
			msg: "d_msg".to_string(),
			stime: now(),
			prio: 4000
		}
	}
}

impl Scheduler {
	pub fn new(name: String, msg: String, time_gap: u64, prio: u16) -> Self {
		Self {
			name: name,
			msg: msg,
			stime: now() + time_gap * 60,
			prio: prio,
		}
	}
}

impl SchedulerList {
	pub fn new() -> Self {
		Self {
			sl: Arc::new(Mutex::new(HashMap::new())),
			uid: Arc::new(Mutex::new(0)),
		}
	}
	fn update(&mut self, id: &ID, name: Option<String>, msg: Option<String>, gap: Option<u64>, prio: Option<u16>) -> bool {
		let mut sl = self.sl.lock().unwrap();
		if let Some(sdr) = (*sl).get_mut(&id) {
			if let Some(n) = name {
				sdr.name = n;
			}
			if let Some(m) = msg {
				sdr.msg = m;
			}
			if let Some(g) = gap {
				sdr.stime = now() + g * 60;
			}
			if let Some(p) = prio {
				sdr.prio = p;
			}

			return true;
		} else {
			return false;
		}
	}
	fn add(&mut self, sd: Scheduler) -> bool {
		let mut sl = self.sl.lock().unwrap();
		let mut uid = self.uid.lock().unwrap();
		
		println!("[num] {uid}");
		match sl.insert(*uid, sd) {
			Some(_) => false,
			None => { 
				*uid += 1;
				true
			}
		}
	}
	fn del(&mut self, index: u64) -> bool {
		let mut sl = self.sl.lock().unwrap();
		sl.remove(&index).is_some()
	}
	fn check(&mut self, time: u64) {
		let mut sl = self.sl.lock().unwrap();
		(*sl).retain(|_, v| if v.stime > time {true} else {println!("{}'s time is up!!", v.name); false});
	}
	fn show(&self) {
		let sd = self.sl.lock().unwrap();
		if sd.is_empty() {
			println!("empty");
		} else {
			for (k, v) in sd.iter() {
				println!("\n[ID]: {}\n{}", k, v);
			}
		}
	}
	fn json(&mut self) -> Vec<String> {
		let sl = self.sl.lock().unwrap();
		let mut set: Vec<String> = Vec::new();
		for (_, v) in &*sl {
			if let Ok(t) = to_string(&v) {
				set.push(t);
			}
		}
		set
	}

	pub fn monitor_map(mut s: SchedulerList, rx: &Receiver<TaskCommand>) {
		loop {
			while let Ok(i) = rx.recv() {
				match i {
					TaskCommand::Add(name, msg, gap, prio) => {
						let sd = Scheduler::new(name, msg, gap, prio);
						match s.add(sd) {
							true => println!("添加成功"),
							false => println!("添加失败"),
						}
					},
					TaskCommand::Remove(id) => {
						match s.del(id) {
							true => (),
							false => println!("没有这个id的数据")
						}
					},
					TaskCommand::Update(id, name, msg, gap, prio) => {
						match s.update(&id, name, msg, gap, prio) {
							true => (),
							false => println!("更新失败")
						}
					},
					TaskCommand::CheckTime(time) => {
						s.check(time);
					}
					TaskCommand::List => {
						s.show();
					},
					TaskCommand::Json => {
						let js = s.json();
						std::thread::sleep(Duration::from_secs(1));
						println!();
						for i in js {
							println!("{i}");
						}
					},
					TaskCommand::Exit=> {
						std::process::exit(0);
					}
				}
			}
		}
	}

}

#[cfg(test)] 
mod sdlr{
	#[test]
	fn test_add() {

	}
	#[test]
	fn test_del() {

	}
	#[test]
	fn test_json() {

	}
}