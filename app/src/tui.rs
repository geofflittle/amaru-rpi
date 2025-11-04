use crate::app::{App, AppAction};
use crate::backends;
use anyhow::Result;
use ratatui::Terminal;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

pub async fn run() -> Result<()> {
    #[cfg(feature = "display_hat")]
    let (backend, input_rx) = backends::display_hat::setup_hardware_and_input()?;
    #[cfg(feature = "simulator")]
    let (backend, input_rx) = backends::simulator::setup_simulator_and_input();
    let mut terminal = Terminal::new(backend)?;
    let startup = Instant::now();
    let mut frame_count = 0;
    let mut last_loop = Instant::now();

    let mut app = App::default();
    let running = Arc::new(AtomicBool::new(true));
    while running.load(Ordering::SeqCst) {
        frame_count += 1;
        let elapsed_since_last_frame = last_loop.elapsed();
        last_loop = Instant::now();

        app.update().await;

        // TODO multiple events
        if let Ok(event) = input_rx.try_recv()
            && let AppAction::Quit = app.handle_input(event)
        {
            running.store(false, Ordering::SeqCst);
        }

        terminal.draw(|frame| {
            app.draw(
                frame,
                frame_count,
                elapsed_since_last_frame,
                startup.elapsed(),
            );
        })?;
    }
    terminal.clear()?;

    Ok(())
}
