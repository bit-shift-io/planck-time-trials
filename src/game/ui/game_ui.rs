use iced::{Element, Theme};
use crate::game::game_state::GameState;
use crate::game::leaderboard::LeaderboardEntry;
use crate::game::ui::hud::hud_view;
use crate::game::ui::leaderboard::leaderboard_view;
use crate::game::ui::name_entry::name_entry_view;


#[derive(Debug, Clone)]
pub struct GameUI {
    pub(crate) fps: i32,
    pub(crate) total_time: f32,
    pub(crate) simulation_time_ms: f32,
    pub(crate) update_time_ms: f32,
    pub(crate) render_time_ms: f32,
    pub(crate) game_state: GameState,
    pub(crate) leaderboard_results: Vec<LeaderboardEntry>,
    pub(crate) name_input: String,
    pub(crate) show_debug_info: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateFps(i32),
    UpdateTime(f32),
    UpdateSimulationTime(f32),
    UpdateUpdateTime(f32),
    UpdateRenderTime(f32),
    UpdateGameState(GameState),
    UpdateLeaderboardResults(Vec<LeaderboardEntry>),
    UpdateNameInput(String),
    UpdateShowDebugInfo(bool),
    SubmitName,
}

impl GameUI {
    pub fn new() -> Self {
        Self {
            fps: 60,
            total_time: 0.0,
            simulation_time_ms: 0.0,
            update_time_ms: 0.0,
            render_time_ms: 0.0,
            game_state: GameState::Playing,
            leaderboard_results: Vec::new(),
            name_input: String::new(),
            show_debug_info: true,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::UpdateFps(fps) => self.fps = fps,
            Message::UpdateTime(time) => self.total_time = time,
            Message::UpdateSimulationTime(time) => self.simulation_time_ms = time,
            Message::UpdateUpdateTime(time) => self.update_time_ms = time,
            Message::UpdateRenderTime(time) => self.render_time_ms = time,
            Message::UpdateGameState(state) => self.game_state = state,
            Message::UpdateLeaderboardResults(results) => self.leaderboard_results = results,
            Message::UpdateNameInput(name) => self.name_input = name,
            Message::UpdateShowDebugInfo(show) => self.show_debug_info = show,
            Message::SubmitName => {} // Handled by Game
        }
    }

    pub fn view(&self) -> Element<'_, Message, Theme, iced::Renderer> {
        match self.game_state {
            GameState::NameEntry => name_entry_view(self),
            GameState::Finished => leaderboard_view(self),
            GameState::Playing => hud_view(self),
        }
    }
}
