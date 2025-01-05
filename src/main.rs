use noob::sdlr::*;
use noob::store::*;
use std::{
	io::{self, Write}, 
	process::exit, 
	sync::{
		mpsc::{channel, Sender, Receiver}, 
	},
	thread::{
		sleep, spawn
	}, 
	time::{
		Duration,
		SystemTime,
		UNIX_EPOCH
	}
};
//use noob::{detail, warn};

fn show_help() {
let st = r#"
	help | ? 		: print this help text
	q | exit | bye | quit 	: quit this program
	add 			: add task to global scheduler list(gsl)
	del			: remove task from gsl
	update			: update task in gsl
	show			: show global schedulers 
"#;
	println!("{}", st);
}

fn add(tx: Sender<TaskCommand>) {
	let mut name = String::new();
	let mut msg = String::new();
	let mut mg_str = String::new();
	let mut prio_str = String::new();
	let mut min_gap: u64 = 0;
	let mut prio: u16 = 0;

	sout("	name:  ");
	io::stdin().read_line(&mut name).unwrap();
	name = name.trim().to_string();
	
	sout("	msg: ");
	io::stdin().read_line(&mut msg).unwrap();
	msg = msg.trim().to_string();
	
	sout("	time_gap(min): ");
	if let Ok(_) = io::stdin().read_line(&mut mg_str) {
		min_gap = match mg_str.trim().parse::<u64>() {
			Ok(t) => t,
			Err(_) => {
				println!("Invalid input");
				return;
			},
		};
	}
	sout("	prio: ");
	if let Ok(_) = io::stdin().read_line(&mut prio_str) {
		prio = match prio_str.trim().parse::<u16>() {
			Ok(t) => t,
			Err(_) => {
				println!("Invalid input");
				return;
			}
		}	
	}

	tx.send(
		TaskCommand::Add(
			name,
			msg,
			min_gap,
			prio
		)
	).unwrap();
	
}

fn del(tx: Sender<TaskCommand>) {
	let mut id_str = String::new();
	let mut id: u64 = 0;

	sout("	id:  ");
	if let Ok(_) = io::stdin().read_line(&mut id_str) {
		id = id_str.trim().parse::<u64>().expect("delete tra failed!");
	}

	tx.send(
		TaskCommand::Remove(id)
	).unwrap();	
}

fn update(tx: Sender<TaskCommand>) {
	let mut id_str = String::new();
	let mut id: u64 = 0;
	let mut name = String::new();
	let mut msg = String::new();
	let mut mg_str = String::new();
	let mut prio_str = String::new();
	let mut min_gap: u64 = 0;
	let mut prio: u16 = 0;

	sout("	id:  ");
	if let Ok(_) = io::stdin().read_line(&mut id_str) {
		id = match id_str.trim().parse::<u64>() {
			Ok(m) => m,
			Err(_) => {
				println!("Invalid input");
				return;
			},
		};
	}

	sout("	name:  ");
	io::stdin().read_line(&mut name).unwrap();
	name = name.trim().to_string();

	sout("	msg: ");
	io::stdin().read_line(&mut msg).unwrap();
	msg = msg.trim().to_string();
	
	sout("	time_gap(min): ");
	if let Ok(_) = io::stdin().read_line(&mut mg_str) {
		min_gap = match mg_str.trim().parse::<u64>() {
			Ok(m) => m,
			Err(_) => 0
		};
	}

	sout("	prio: ");
	if let Ok(_) = io::stdin().read_line(&mut prio_str) {
		prio = match prio_str.trim().parse::<u16>() {
			Ok(p) => p,
			Err(_) => 0
		};
	}

	tx.send(
		TaskCommand::Update(
			id, 
			if name.len() > 0 {Some(name)} else {None},
			if msg.len() > 0 {Some(msg)} else {None},
			if min_gap != 0 {Some(min_gap)} else {None},
			if prio != 0 {Some(prio)} else {None}
		)
	).unwrap();
}

fn show(tx: Sender<TaskCommand>) {
	tx.send(
		TaskCommand::List
	).unwrap();
}

// store用sdr的tx写回json数据，用于构建或者新增SchedulerList
// sdr用store的tx传输Vec<String>，给store去写入文件
fn to_json(tx: Sender<TaskCommand>) {
	tx.send(
		TaskCommand::Json
	).unwrap();
}

fn cmd(il: &String, tx: Sender<TaskCommand>) {
	let s :&str = &il.trim().to_lowercase();
	match s {
		"?" | "help" | "h" => {
			show_help()
		},
		"exit" | "quit" | "bye" | "q" => {
			println!("Bye");
			exit(0)
		},
		"add" => {
			add(tx);
		},
		"del" => {
			del(tx);
		}
		"update" => {
			update(tx);
		}
		"show" => {
			show(tx);
		},
		"json" => {
			to_json(tx);
		}
		_ => {
			println!("nothing happend..")
		}
	}
}

fn sout(st: &str) {
	print!("{}", st);
	io::stdout().flush().unwrap();
}

fn cmd_line() {
	print!("[>] ");
	io::stdout().flush().unwrap();
}

// timer
fn manager_thread_func(tx: Sender<TaskCommand>) {
	loop {
		sleep(Duration::from_secs(1));
		tx.send(
			TaskCommand::CheckTime(
				SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.unwrap()
				.as_secs()
			)
		).unwrap();
	};
}

// channel checker
fn channel_checker(sdr: SchedulerList, rx: Receiver<TaskCommand>) {
	SchedulerList::monitor_map(sdr, &rx);
}

fn main() {
	let (tx, rx): (Sender<TaskCommand>, Receiver<TaskCommand>) = channel();
	let mut line = String::new();
	let sdr = SchedulerList::new();
	let g_file = Store::new("scheduler.json");
	cmd_line();
	
	// monitor thread
	spawn(move || {
		channel_checker(sdr, rx);
	});

	// time thread
	let tx_m = tx.clone();
	spawn(move || {
		manager_thread_func(tx_m);
	});

	// user input main thread
	loop {
		if let Ok(_) = io::stdin().read_line(&mut line) {
			let tx = tx.clone();
			cmd(&line, tx);
			line.clear();

		} else {
			println!("read failed");
		}
		cmd_line();
	}
}
