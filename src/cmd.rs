pub struct cmd{}

impl cmd {
    fn show_help(&self) {
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

    
    pub fn run() {

    }
}