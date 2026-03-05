mod app;
mod config;
mod session;
mod tree;
mod ui;

use std::io;

use crossterm::{
    event::{self, Event},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let config = config::Config::load();
    let mut app = app::App::new(config);

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if let Event::Key(key) = event::read()? {
            app.handle_key(key);
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    if let Some(cmd) = app.resume_command.take() {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let err = std::process::Command::new("sh").arg("-c").arg(&cmd).exec();
            eprintln!("exec failed: {err}");
        }

        #[cfg(not(unix))]
        {
            if let Err(e) = std::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .status()
            {
                eprintln!("command failed: {e}");
            }
        }
    }

    Ok(())
}
