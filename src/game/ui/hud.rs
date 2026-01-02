use iced::widget::{column, text, container};
use iced::{Color, Element, Length, Theme, Alignment};
use super::game_ui::{Message, GameUI};

pub fn hud_view(ui: &GameUI) -> Element<'_, Message, Theme, iced::Renderer> {
    container(
        column![
            text(format!("FPS: {}", ui.fps))
                .size(20)
                .color(Color::WHITE),
            text(format!("Time: {:.2}s", ui.total_time))
                .size(20)
                .color(Color::WHITE),
            text(format!("Sim Time: {:.2}ms", ui.simulation_time_ms))
                .size(20)
                .color(Color::WHITE),
        ]
        .padding(10)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Start)
    .align_y(Alignment::Start)
    .into()
}
