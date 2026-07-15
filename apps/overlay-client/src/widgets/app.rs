//! iced UI for the overlay. Reads RoomState, produces an Element.
//!
//! SPIKE: intentionally minimal — coloured header + one row per participant.
//! Uses the iced sub-crates directly (no umbrella `iced`) so the Element's
//! Renderer generic is `iced_wgpu::Renderer` and headless rendering works
//! without a fallback wrapper.

use iced_core::{
    Alignment, Background, Border, Color, Element, Length, Theme,
    alignment::Vertical,
};
use iced_widget::{Column, column, container, row, text};

use crate::voice_state::{ActiveRoom, Participant, RoomState};

pub type Renderer = iced_wgpu::Renderer;

#[derive(Debug, Clone, Copy)]
pub enum Message {}

pub fn view(state: &RoomState) -> Element<'_, Message, Theme, Renderer> {
    let content: Element<Message, Theme, Renderer> = match state.active_room.as_ref() {
        Some(room) => room_view(room).into(),
        None => idle_view().into(),
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(12)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.55))),
            border: Border {
                radius: 8.0.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
}

fn idle_view() -> Column<'static, Message, Theme, Renderer> {
    column![
        text("Hermes").size(20).color(Color::WHITE),
        text("not in a voice room")
            .size(12)
            .color(Color::from_rgb(0.7, 0.7, 0.7)),
    ]
    .spacing(4)
}

fn room_view(room: &ActiveRoom) -> Column<'_, Message, Theme, Renderer> {
    let header = column![
        text(format!("# {}", room.name))
            .size(18)
            .color(Color::WHITE),
        text(format!("{} in call", room.participants.len()))
            .size(11)
            .color(Color::from_rgb(0.7, 0.7, 0.7)),
    ]
    .spacing(2);

    let mut list: Column<Message, Theme, Renderer> = Column::new().spacing(4);
    for p in &room.participants {
        list = list.push(participant_row(p));
    }

    column![header, list].spacing(10)
}

fn participant_row(p: &Participant) -> Element<'_, Message, Theme, Renderer> {
    let dot_color = if p.speaking {
        Color::from_rgb(0.35, 0.85, 0.4)
    } else if p.muted {
        Color::from_rgb(0.85, 0.35, 0.35)
    } else {
        Color::from_rgb(0.5, 0.5, 0.5)
    };

    row![
        container(text(" ").size(10))
            .width(Length::Fixed(10.0))
            .height(Length::Fixed(10.0))
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(dot_color)),
                border: Border {
                    radius: 5.0.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }),
        text(p.display_name.clone()).size(14).color(Color::WHITE),
    ]
    .spacing(8)
    .align_y(Alignment::from(Vertical::Center))
    .into()
}
