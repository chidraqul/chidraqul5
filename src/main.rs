//importing in execute! macro
extern crate crossterm;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use terminal_size::{terminal_size, Height, Width};

use std::process;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::{thread, time};

const WORLD_HEIGHT: i32 = 10;
const WORLD_WIDTH: i32 = 20;

struct Player {
    x: i32,
    y: i32,
}

fn die(player: &mut Player) {
    player.x = WORLD_WIDTH / 2;
    player.y = 0;
}

fn quit() {
    disable_raw_mode().unwrap();
    process::exit(1);
}

fn render(player: &mut Player) {
    let size = terminal_size();
    if let Some((Width(w), Height(h))) = size {
        let width: usize = (w - 2).into();
        for _y in 0..(h - 1) {
            println!("|{}|\r", format!("{:w$}", "", w = width));
        }
    } else {
        println!("Unable to get terminal size");
        quit();
    }
    println!("x={} y={} | ctrl+q to quit\r", player.x, player.y);
}

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        match read().unwrap() {
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::NONE,
            }) => tx.send("d".to_string()).unwrap(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::NONE,
            }) => tx.send("a".to_string()).unwrap(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
            }) => tx.send("q".to_string()).unwrap(),
            _ => (),
        }
    });
    rx
}

fn tick(player: &mut Player) {
    // player.y += 1;
    if player.x > WORLD_WIDTH || player.x < 0 {
        die(player);
    }
    if player.y > WORLD_HEIGHT || player.y < 0 {
        die(player);
    }
    render(player);
    thread::sleep(time::Duration::from_millis(10));
}

fn got_key(key: String, player: &mut Player) {
    if key == "q" {
        quit();
    } else if key == "d" {
        player.x += 1;
    } else if key == "a" {
        player.x -= 1;
    }
}

fn main() {
    enable_raw_mode().unwrap();
    let mut player = Player {
        x: WORLD_WIDTH / 2,
        y: 0,
    };
    let stdin_channel = spawn_stdin_channel();
    loop {
        tick(&mut player);
        match stdin_channel.try_recv() {
            Ok(key) => got_key(key, &mut player),
            Err(TryRecvError::Empty) => continue,
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
    }
}
