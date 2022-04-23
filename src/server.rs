use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

pub mod shared;
use shared::player::Player;

// TODO: move code below to gamelogic.rs

const WORLD_HEIGHT: i16 = 20;
const WORLD_WIDTH: i16 = 40;

fn die(player: &mut Player) {
    player.x = WORLD_WIDTH / 2;
    player.y = 1;
}

fn tick(player: &mut Player) {
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
}

// TODO: move code below to network.rs

fn disconnect(stream: TcpStream, err: String) {
    println!("Disconnected ({})", err);
    match stream.shutdown(Shutdown::Both) {
        _ => (), // This is probably not idiomatic rust ;D
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    let mut player = Player {
        x: WORLD_WIDTH / 2,
        y: 1,
        vel_y: 0,
    };
    while match stream.read(&mut data) {
        Ok(_size) => {
            tick(&mut player);
            let data_str = std::str::from_utf8(&data).unwrap().to_string();
            println!("got data: {}", data_str);
            if data_str.chars().nth(0).unwrap() == '1' {
                player.x -= 1;
            }
            let reply = format!("{:0>3}{:0>3}", player.x, player.y);
            match stream.write(reply.as_bytes()) {
                Ok(_) => (),
                Err(err) => {
                    disconnect(stream, err.to_string());
                    return;
                }
            }
            true
        }
        Err(_) => {
            match stream.peer_addr() {
                Ok(err) => println!("Disconnect: {}", err),
                Err(err) => println!("Disconnect (err): {}", err),
            }
            disconnect(stream, "TODO".to_string());
            return;
        }
    } {}
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:5051").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 5051");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || {
                    // connection succeeded
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
}
