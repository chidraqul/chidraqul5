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
use std::sync::mpsc::TryRecvError;
use std::{thread, time};

#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::*;

const WORLD_HEIGHT: i16 = 20;
const WORLD_WIDTH: i16 = 40;

struct Player {
    x: i16,
    y: i16,
    vel_y: i16,
}

fn die(player: &mut Player) {
    player.x = WORLD_WIDTH / 2;
    player.y = 0;
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
    if player.vel_y > 0 {
        player.vel_y -= 1;
    }
    if player.vel_y < 0 {
        player.vel_y += 1;
    }
    if player.y < WORLD_HEIGHT {
        player.vel_y += 1;
    }
    player.y += player.vel_y;
    if player.x > WORLD_WIDTH || player.x < 0 {
        die(player);
    }
    if player.y > WORLD_HEIGHT || player.y < 0 {
        die(player);
    }
    if let Err(e) = render(player, stdout) {
        println!("{}", e);
        quit();
    }
    thread::sleep(time::Duration::from_millis(10));
}

fn got_key(key: String, player: &mut Player) {
    if key == "q" {
        quit();
    } else if key == "d" {
        player.x += 1;
    } else if key == "a" {
        player.x -= 1;
    } else if key == " " {
        player.vel_y -= 6;
    }
}

fn network() {
    match TcpStream::connect("localhost:5051") {
        Ok(mut stream) => {
            info!("Successfully connected to server in port 5051");

            let msg = b"Hello!";

            stream.write(msg).unwrap();
            info!("Sent Hello, awaiting reply...");

            let mut data = [0 as u8; 6]; // using 6 byte buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if &data == msg {
                        info!("Reply is ok!");
                    } else {
                        let text = from_utf8(&data).unwrap();
                        info!("Unexpected reply: {}", text);
                    }
                }
                Err(e) => {
                    info!("Failed to receive data: {}", e);
                }
            }
        }
        Err(e) => {
            info!("Failed to connect: {}", e);
        }
    }
    info!("Terminated.");
}

fn main() {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        File::create("client.log").unwrap(),
    )])
    .unwrap();
    network();
    enable_raw_mode().unwrap();
    let mut player = Player {
        x: WORLD_WIDTH / 2,
        y: 0,
        vel_y: 0,
    };
    let stdin_channel = spawn_stdin_channel();
    let mut stdout = stdout();
    loop {
        tick(&mut player, &mut stdout);
        match stdin_channel.try_recv() {
            Ok(key) => got_key(key, &mut player),
            Err(TryRecvError::Empty) => continue,
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
    }
}
