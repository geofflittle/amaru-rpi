use crate::actions::handle_action;
use crate::app::{App, AppAction, AppEvent};
use crate::backends;
use anyhow::Result;
use ratatui::Terminal;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub async fn run() -> Result<()> {
    let (tx, input_rx) = std::sync::mpsc::channel();

    #[cfg(feature = "display_hat")]
    let backend = backends::display_hat::setup(tx.clone())?;
    #[cfg(feature = "simulator")]
    let backend = backends::simulator::setup_simulator_and_input(tx.clone());
    #[cfg(target_os = "linux")]
    {
        // Only spawn if we are on Linux, otherwise this crashes
        crate::inputs::evdev::spawn_listener(tx.clone())?;
    };

    let mut terminal = Terminal::new(backend)?;
    let mut app = App::default();
    let running = Arc::new(AtomicBool::new(true));
    let mut events: Vec<AppEvent> = Vec::with_capacity(4);
    while running.load(Ordering::SeqCst) {
        events.push(AppEvent::Tick);
        while let Ok(event) = input_rx.try_recv() {
            events.push(AppEvent::Input(event));
        }

        for event in events.drain(..) {
            let actions = app.update(event);
            for action in actions {
                if action == AppAction::Quit {
                    running.store(false, Ordering::SeqCst);
                    break;
                }
                handle_action(&mut app, action).await
            }
        }

        if !running.load(Ordering::SeqCst) {
            break;
        }

        terminal.draw(|frame| {
            app.draw(frame);
        })?;
    }
    terminal.clear()?;

    Ok(())
}
