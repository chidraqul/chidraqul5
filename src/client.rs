//importing in execute! macro
extern crate crossterm;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use terminal_size::{terminal_size, Height, Width};

use std::net::TcpStream;
use std::str::from_utf8;

use std::fs::File;
use std::io::{stdout, Read, Write};
use std::process;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::{thread, time};

#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::*;

pub mod shared;
use shared::player::Player;

const WORLD_WIDTH: i16 = 40;

struct Controls {
    left: bool,
    right: bool,
    jump: bool,
}

fn quit() {
    disable_raw_mode().unwrap();
    process::exit(1);
}

fn render(player: &mut Player, stdout: &mut std::io::Stdout) -> Result<()> {
    let size = terminal_size();
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    if let Some((Width(_w), Height(h))) = size {
        // let width: usize = (w - 2).into();
        // for _y in 0..(h - 1) {
        //     println!("|{}|\r", format!("{:w$}", "", w = width));
        // }
        stdout
            .queue(cursor::MoveTo(
                player.x.try_into().unwrap(),
                player.y.try_into().unwrap(),
            ))?
            .queue(style::PrintStyledContent("█".magenta()))?
            .queue(cursor::MoveTo(0, h))?;
        println!(
            "x={} y={} | ctrl+q to quit | A, D an SPACE to move\r",
            player.x, player.y
        );
    } else {
        println!("Unable to get terminal size");
        quit();
    }
    stdout.flush()?;
    Ok(())
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
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
            }) => tx.send(" ".to_string()).unwrap(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
            }) => tx.send("q".to_string()).unwrap(),
            _ => (),
        }
    });
    rx
}

fn tick(player: &mut Player, stdout: &mut std::io::Stdout) {
    if let Err(e) = render(player, stdout) {
        println!("{}", e);
        quit();
    }
    thread::sleep(time::Duration::from_millis(10));
}

fn got_key(key: String, controls: &mut Controls) {
    if key == "q" {
        quit();
    } else if key == "a" {
        controls.left = true;
    } else if key == "d" {
        controls.right = true;
    } else if key == " " {
        controls.jump = true;
    }
}

fn got_data(data: String, player: &mut Player) {
    info!("got data {}", data);
    // let mut slices: Vec<&str> = Vec::new();
    // slices.push(&data[..3]);
    // slices.push(&data[3..]);
    // for slice in slices {
    //     info!("slice: {}", slice);
    // }
    match data[..3].parse::<i16>() {
        Ok(x) => player.x = x,
        Err(e) => warn!("Got invalid data='{}' err='{}'", data, e),
    }
    match data[3..].parse::<i16>() {
        Ok(y) => player.y = y,
        Err(e) => warn!("Got invalid data='{}' err='{}'", data, e),
    }
}

fn spawn_network_channel() -> (Receiver<String>, Sender<String>) {
    let (tx, rx) = mpsc::channel::<String>();
    let (in_tx, in_rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        match TcpStream::connect("localhost:5051") {
            Ok(mut stream) => {
                info!("Successfully connected to server in port 5051");

                loop {
                    let msg = b"Hello!";
                    let mut sent = false;
                    match in_rx.try_recv() {
                        Ok(input) => {
                            info!("Thread got data {}", input);
                            stream.write(input.as_bytes()).unwrap();
                            sent = true;
                        }
                        Err(TryRecvError::Empty) => (),
                        Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
                    }
                    if !sent {
                        stream.write(msg).unwrap();
                    }
                    // info!("Sent Hello, awaiting reply...");

                    let mut data = [0 as u8; 6]; // using 6 byte buffer
                    match stream.read_exact(&mut data) {
                        Ok(_) => {
                            let data_str = from_utf8(&data).unwrap().to_string();
                            info!("Reply is ok! ({})", data_str);
                            tx.send(data_str).unwrap();
                        }
                        Err(e) => {
                            info!("Failed to receive data: {}", e);
                        }
                    }
                    thread::sleep(time::Duration::from_millis(10));
                }
            }
            Err(e) => {
                info!("Failed to connect: {}", e);
            }
        }
    });
    (rx, in_tx)
}

fn controls_to_network(controls: &Controls) -> String {
    return format!(
        "{}{}{}000",
        controls.left as i8, controls.right as i8, controls.jump as i8
    );
}

fn main() {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        File::create("client.log").unwrap(),
    )])
    .unwrap();
    enable_raw_mode().unwrap();
    let mut controls = Controls {
        left: false,
        right: false,
        jump: false,
    };
    let mut player = Player {
        id: 0,
        x: WORLD_WIDTH / 2,
        y: 1,
        vel_y: 0,
    };
    let stdin_channel = spawn_stdin_channel();
    let (network_channel, in_tx) = spawn_network_channel();
    let mut stdout = stdout();
    loop {
        tick(&mut player, &mut stdout);
        in_tx.send(controls_to_network(&controls)).unwrap();
        match stdin_channel.try_recv() {
            Ok(key) => got_key(key, &mut controls),
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
        match network_channel.try_recv() {
            Ok(data) => got_data(data, &mut player),
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
    }
}
