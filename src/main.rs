use crossterm::ExecutableCommand;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use invaders::frame::Drawable;
use invaders::{frame, player::Player, render};
use rusty_audio::Audio;
use std::error::Error;
use std::sync::mpsc;
use std::time::Duration;
use std::{io, thread};

fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    audio.add("explosion", "audio/original/explode.wav");
    audio.add("lose", "audio/original/lose.wav");
    audio.add("move", "audio/original/move.wav");
    audio.add("pew", "audio/original/pew.wav");
    audio.add("startup", "audio/original/startup.wav");
    audio.add("win", "audio/original/win.wav");

    audio.play("startup");

    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // Game loop
    let mut player = Player::new();
    'game_loop: loop {
        // Per-frame init
        let mut curr_frame = frame::new_frame();

        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char('q') | KeyCode::Esc => {
                        audio.play("lose");
                        break 'game_loop;
                    }
                    _ => continue,
                }
            }
        }

        // Draw & Render
        player.draw(&mut curr_frame);
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
