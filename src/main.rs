mod config;
mod datapoint;
mod scanner;
mod ui;

use anyhow::Result;
use clap::Parser;
use config::Config;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use scanner::Scanner;
use std::{
    io,
    time::Duration,
};
use tokio::{sync::Mutex, time::interval};
use std::sync::Arc;
use ui::App;

#[derive(Parser, Debug)]
#[command(name = "datapoint_tui")]
#[command(about = "A TUI dashboard for monitoring server datapoints", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
    
    #[arg(short, long, help = "Enable debug logging to file")]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logger if debug flag is set
    if args.debug {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .target(env_logger::Target::Pipe(Box::new(
                std::fs::File::create("datapoint_tui_debug.log")?
            )))
            .init();
        log::info!("Debug logging enabled");
    }
    
    let config = Config::load(&args.config)?;
    
    let server_info = format!("{}://{}:{}", config.server.protocol, config.server.host, config.server.port);
    let scan_interval = config.scan_interval_ms;
    
    let scanner = Arc::new(Mutex::new(Scanner::new(config)));
    
    let scanner_clone = scanner.clone();
    tokio::spawn(async move {
        // Wait a bit before first scan to let UI initialize
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        loop {
            {
                let mut scanner = scanner_clone.lock().await;
                // Run one scan, then release the lock
                let _ = scanner.scan_once().await;
            }
            // Wait for scan interval before next scan
            tokio::time::sleep(Duration::from_millis(scan_interval)).await;
        }
    });
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(server_info, scan_interval);
    
    let res = run_app(&mut terminal, &mut app, scanner).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    scanner: Arc<Mutex<Scanner>>,
) -> Result<()> {
    let mut update_interval = interval(Duration::from_millis(100));
    
    // Draw initial UI
    terminal.draw(|f| ui::draw(f, app)).map_err(|e| anyhow::anyhow!("{}", e))?;
    
    loop {
        // Check for key events first (non-blocking)
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => return Ok(()),
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    _ => {}
                }
            }
        }

        tokio::select! {
            _ = update_interval.tick() => {
                // Try to get datapoints with timeout
                let datapoints_result = tokio::time::timeout(
                    Duration::from_millis(50),
                    async {
                        let scanner_guard = scanner.lock().await;
                        scanner_guard.get_datapoints().to_vec()
                    }
                ).await;
                
                if let Ok(datapoints) = datapoints_result {
                    app.update_datapoints(datapoints);
                }
                
                // Always redraw UI even if we couldn't get data
                terminal.draw(|f| ui::draw(f, app)).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                // Just wait, key handling is done above
            }
        }
    }
}
