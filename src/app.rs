use std::{collections::HashMap, io};

type Identifier = u32;

const START_BUTTON_ID: u32 = 6969;

use color_eyre::owo_colors::OwoColorize;
use rand::{rng, seq::IndexedRandom};
use ratatui::{
    buffer::Buffer, crossterm::event::{self, Event, KeyCode, KeyEvent}, layout::{Constraint, Flex, Layout, Rect}, style::{Style, Stylize}, text::{Text, ToText}, widgets::Widget, DefaultTerminal, Frame
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::db::{AlgDB, AlgEntry, AlgSet, Movement};

#[derive(Debug)]
pub struct App<'a> {
    pub page: AppPage<'a>,
    pub db: &'a AlgDB,
    pub exit: bool,
}

impl<'a> App<'a> {
    pub fn new(db: &'a AlgDB) -> App<'a> {
        fn parse_entries<'a>(entries: &'a [AlgEntry], id: &mut u32, algset_map: &mut HashMap<Identifier, AlgInfo<'a>>) {
            for entry in entries {
                match entry {
                    AlgEntry::Group(_name, entries) => {
                        *id += 1;
                        parse_entries(entries, id, algset_map);
                    }
                    AlgEntry::Algs(_name, algs) => {
                        let info = AlgInfo {
                            algset: algs,
                            enabled: false,
                        };
                        algset_map.insert(*id, info);
                    }
                }
                *id += 1;
            }
        }

        let mut algset_map: HashMap<Identifier, AlgInfo> = HashMap::new();
        let mut id: u32 = 0;
        parse_entries(&db.entries, &mut id, &mut algset_map);

        let mut state = TreeState::default();
        state.select(vec![0]);

        let page = AppPage::Setup {
            state,
            algset_map,
            db,
        };

        App {
            db,
            page: page,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) {
        loop {
            terminal.draw(|frame| self.draw(frame)).unwrap();

            if let Event::Key(key) = event::read().unwrap() {
                unsafe {
                    let ptr = self as *mut App<'a>;
                    self.page.handle_key(ptr.as_mut().unwrap(), key);
                }
            }
            if self.exit {
                break;
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        self.page.draw(frame);
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

#[derive(Debug)]
pub struct AlgInfo<'a> {
    pub algset: &'a AlgSet,
    pub enabled: bool,
}

#[derive(Debug)]
enum AppPage<'a> {
    Setup {
        state: TreeState<Identifier>,
        db: &'a AlgDB,
        algset_map: HashMap<Identifier, AlgInfo<'a>>,
    },
    Train {
        algs: Vec<&'a AlgSet>,
        scramble: String,
    },
}

pub fn get_scramble<'a>(algsets: &'a [&'a AlgSet]) -> String {
    let mut movements: Vec<&'a [Movement]> = Vec::new();

    for algset in algsets {
        for alg in algset.algs.iter() {
            movements.push(&alg);
        }
    }

    let movements = movements.choose(&mut rng()).unwrap();
    
    let mut text = String::new();
    
    for (i, movement) in movements.iter().rev().enumerate() {
        text.push_str(movement.inv().as_text());
        if i < movements.len()-1 {
            text.push(' ');
        }
    }

    text
}

impl<'a> AppPage<'a> {
    pub fn handle_key(&mut self, app: &mut App<'a>, key: KeyEvent) {
        if let KeyCode::Char('q') = key.code {
            app.exit = true;
        }
        match self {
            AppPage::Setup { state, algset_map, .. } => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.key_up();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        state.key_down();
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        // state.toggle_selected();
                        state.toggle_selected();
                        if let Some(identifier) = state.selected().last() {
                            if let Some(algset) = algset_map.get_mut(identifier) {
                                algset.enabled = !algset.enabled;
                            } else if *identifier == START_BUTTON_ID {
                                let algs: Vec<&'a AlgSet> = algset_map.values().filter(|info| info.enabled).map(|info| info.algset).collect();
                                if algs.len() > 0 {
                                    let scramble = get_scramble(&algs);
                                    app.page = AppPage::Train {
                                        algs,
                                        scramble,
                                    };
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            AppPage::Train {scramble, algs, ..} => {
                match key.code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        *scramble = get_scramble(&algs);
                    }
                    _ => {},
                }
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        match self {
            AppPage::Setup { state, db, algset_map, .. } => {
                fn parse_entries<'a>(entries: &'a [AlgEntry], id: &mut u32, algset_map: &mut HashMap<Identifier, AlgInfo<'a>>) -> Vec<TreeItem<'a, Identifier>> {
                    let mut ret_items: Vec<TreeItem<Identifier>> = Vec::new();
                    for entry in entries {
                        match entry {
                            AlgEntry::Group(name, entries) => {
                                let mut group = TreeItem::new(*id, name.clone(), vec![]).unwrap();
                                *id += 1;
                                let items = parse_entries(entries, id, algset_map);
                                for item in items {
                                    group.add_child(item).unwrap();
                                }
                                ret_items.push(group);
                            }
                            AlgEntry::Algs(name, algs) => {
                                let info = AlgInfo {
                                    algset: algs,
                                    enabled: false,
                                };

                                let mut text = format!("|-- {}", name.clone());
                                if !algset_map.get(id).unwrap().enabled {
                                    text = name.clone();
                                }
                                
                                let item = TreeItem::new_leaf(*id, text);
                                ret_items.push(item);
                            }
                        }
                        *id += 1;
                    }
                    ret_items
                }

                let mut entries = parse_entries(&db.entries, &mut 0, algset_map);
                let start_button = TreeItem::new_leaf(START_BUTTON_ID, "Start");
                entries.push(start_button);

                let widget = Tree::new(&entries).unwrap().highlight_symbol("> ");
                frame.render_stateful_widget(widget, frame.area(), state);
            }
            AppPage::Train {scramble, ..} => {
                let text = scramble.to_text();

                text.render(frame.area(), frame.buffer_mut());
            }
        }
    }
}
