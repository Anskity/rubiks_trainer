use std::env;
use std::path::PathBuf;
use rubiks_trainer::app::App;
use rubiks_trainer::db::AlgDB;
fn main() {
    let args: Vec<String> = env::args().collect();
    let alg_dir: String = args.get(1).map(|str| str.clone()).unwrap_or(".".to_string());
    let db = AlgDB::load(PathBuf::from(alg_dir));

    let mut app = App::new(&db);
    color_eyre::install().unwrap();
    let mut term = ratatui::init();
    let _result = app.run(&mut term);
    ratatui::restore();
}
