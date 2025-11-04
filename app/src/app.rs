use crate::button::{ButtonId, ButtonPress, InputEvent};
use crate::network_status::NetworkStatusCache;
use crate::screen_flow::ScreenFlow;
use crate::screens::State;
use ratatui::prelude::*;
use std::time::Duration;

pub enum AppAction {
    Quit,
    None,
}

// The App struct now holds all the state
pub struct App {
    screen_flow: ScreenFlow,
    connectivity_cache: NetworkStatusCache,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen_flow: ScreenFlow::default(),
            connectivity_cache: NetworkStatusCache::new(Duration::from_secs(5)),
        }
    }
}

impl App {
    pub async fn update(&mut self) {
        // Update network status
        self.connectivity_cache.get().await;
    }

    pub fn handle_input(&mut self, event: InputEvent) -> AppAction {
        if !self.screen_flow.handle_input(event)
            && let (ButtonId::B, ButtonPress::Double) = (event.id, event.press_type)
        {
            return AppAction::Quit;
        }
        AppAction::None
    }

    pub fn draw(
        &mut self,
        frame: &mut Frame,
        frame_count: u64,
        last_frame: Duration,
        startup: Duration,
    ) {
        let state = State::new(
            frame_count,
            last_frame,
            startup,
            self.connectivity_cache.last_result,
        );
        self.screen_flow.display(state, frame);
    }
}
