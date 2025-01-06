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
	save			: save gsl to file as json
	load			: load gsl from json file
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

// map -> store
fn to_json(tx: Sender<TaskCommand>, stx: Sender<StoreToken>) {
	tx.send(
		TaskCommand::ToJson(stx)
	).unwrap();
}

// store -> map
fn from_json(tx: Sender<TaskCommand>, stx: Sender<StoreToken>) {
	stx.send(
		StoreToken::Read(tx)
	).unwrap();
}

// 
fn cmd(il: &String, tx: Sender<TaskCommand>, stx: Sender<StoreToken>) {
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
		"save" => {
			to_json(tx, stx);
		},
		"load" => {
			from_json(tx, stx);
		},
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

fn store_checker(st: Store, rx:Receiver<StoreToken>) {
	Store::monitor_store(st, &rx);
}

fn main() {
	let (tx, rx) = channel::<TaskCommand>();
	let (stx, srx)= channel::<StoreToken>();
	let mut line = String::new();
	let sdr = SchedulerList::new();
	let g_file = Store::new("scheduler.json");
	cmd_line();
	
	// scheduler list monitor thread
	spawn(move || {
		channel_checker(sdr, rx);
	});

	// monitor store thread
	spawn(move || {
		store_checker(g_file, srx);
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
			let stx = stx.clone();
			cmd(&line, tx, stx);
			line.clear();
		} else {
			println!("read failed");
		}
		cmd_line();
	}
}
