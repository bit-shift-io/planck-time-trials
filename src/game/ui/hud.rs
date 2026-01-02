use iced::widget::{column, text, container};
use iced::{Color, Element, Length, Theme, Alignment};
use super::game_ui::{Message, GameUI};

pub fn hud_view(ui: &GameUI) -> Element<'_, Message, Theme, iced::Renderer> {
    let mut content = column![].padding(10).spacing(2);

    content = content.push(
        text(format!("Time: {:.2}s", ui.total_time))
            .size(18)
            .color(Color::WHITE)
    );

    if ui.show_debug_info {
        content = content.push(
            text(format!("FPS: {}", ui.fps))
                .size(15)
                .color(Color::WHITE)
        );
        content = content.push(
            text(format!("Update: {:.2}ms", ui.update_time_ms))
                .size(15)
                .color(Color::WHITE)
        );
        content = content.push(
            text(format!("Sim: {:.2}ms", ui.simulation_time_ms))
                .size(15)
                .color(Color::WHITE)
        );
        content = content.push(
            text(format!("Render: {:.2}ms", ui.render_time_ms))
                .size(15)
                .color(Color::WHITE)
        );
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Start)
        .align_y(Alignment::Start)
        .into()
}
