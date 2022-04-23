use std::process;
use std::{thread, time};
use terminal_size::{Width, Height, terminal_size};

const WORLD_HEIGHT: i32 = 10;
const WORLD_WIDTH: i32 = 20;

struct Player {
    x: i32,
    y: i32,
}

fn die(player: &mut Player) {
    player.x = 0;
    player.y = 0;
}

fn render(player: &mut Player) {
    let size = terminal_size();
    if let Some((Width(w), Height(h))) = size {
            println!("Your terminal is {} cols wide and {} lines tall", w, h);
            let width: usize = (w - 2).into();
            for _y in 0..(h - 1) {
                println!("|{}|", format!("{:w$}", "", w=width));
            }
    } else {
            println!("Unable to get terminal size");
            process::exit(1);
    }
    println!("x={} y={}", player.x, player.y);
}

fn tick(player: &mut Player) {
    player.x += 1;
    if player.x > WORLD_WIDTH || player.x < 0 {
        die(player);
    }
    if player.y > WORLD_HEIGHT || player.y < 0 {
        die(player);
    }
    render(player);
    thread::sleep(time::Duration::from_millis(10));
}

fn main() {
    let mut player = Player { x: 0, y: 0 };
    loop {
        tick(&mut player);
    }
}

