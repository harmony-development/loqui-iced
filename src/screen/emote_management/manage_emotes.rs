use super::super::Message as TopLevelMessage;
use client::{
    error::ClientError,
    harmony_rust_sdk::{
        api::{emote::Emote, rest::FileId},
        client::api::emote::{AddEmoteToPack, DeleteEmoteFromPack},
    },
};
use iced::Tooltip;
use iced_aw::Card;

use crate::{
    client::Client,
    component::*,
    label, label_button, length,
    screen::{map_to_nothing, select_upload_files, ClientExt},
    style::{Theme, PADDING, SPACING},
};

#[derive(Debug, Clone)]
pub enum Message {
    GoBack,
    NewEmoteNameChanged(String),
    UploadEmote,
    DeleteEmote(String),
    CopyToClipboard(String),
}

#[derive(Debug, Clone, Default)]
pub struct ManageEmotesModal {
    but_states: Vec<(button::State, button::State, button::State)>,
    emotes_state: scrollable::State,
    new_emote_name_state: text_input::State,
    new_emote_upload_state: button::State,
    new_emote_name: String,
    pub pack_id: u64,
}

impl ManageEmotesModal {
    pub fn view<'a>(&'a mut self, theme: &Theme, client: &Client, thumbnails: &ThumbnailCache) -> Element<'a, Message> {
        let mut emotes = Scrollable::new(&mut self.emotes_state)
            .style(theme)
            .align_items(Align::Start)
            .spacing(SPACING)
            .padding(PADDING / 2)
            .width(length!(+))
            .height(length!(+));

        if let Some(pack) = client.emote_packs.get(&self.pack_id) {
            self.but_states.resize_with(pack.emotes.len(), Default::default);
            for ((image_id, name), (copy_name_state, copy_id_state, delete_state)) in
                pack.emotes.iter().zip(self.but_states.iter_mut())
            {
                let file_id = FileId::Id(image_id.to_string());
                let emote_image: Element<Message> = thumbnails
                    .emotes
                    .get(&file_id)
                    .or_else(|| thumbnails.thumbnails.get(&file_id))
                    .map_or_else(
                        || label!(image_id).into(),
                        |handle| {
                            Image::new(handle.clone())
                                .height(length!(= 48))
                                .width(length!(= 48))
                                .into()
                        },
                    );
                emotes = emotes.push(
                    Container::new(
                        Row::with_children(vec![
                            Tooltip::new(
                                Button::new(copy_id_state, emote_image)
                                    .on_press(Message::CopyToClipboard(image_id.to_string()))
                                    .style(theme),
                                "Copy image ID to clipboard",
                                iced::tooltip::Position::Top,
                            )
                            .style(theme)
                            .into(),
                            Tooltip::new(
                                label_button!(copy_name_state, name)
                                    .on_press(Message::CopyToClipboard(name.to_string()))
                                    .style(theme),
                                "Copy name to clipboard",
                                iced::tooltip::Position::Top,
                            )
                            .style(theme)
                            .into(),
                            space!(w+).into(),
                            Tooltip::new(
                                Button::new(delete_state, icon(Icon::Trash))
                                    .style(theme)
                                    .on_press(Message::DeleteEmote(name.to_string())),
                                "Delete emote",
                                iced::tooltip::Position::Top,
                            )
                            .style(theme)
                            .into(),
                        ])
                        .align_items(Align::Center)
                        .spacing(SPACING),
                    )
                    .padding(PADDING / 2)
                    .style(theme)
                    .center_x()
                    .center_y(),
                );
            }
        }

        let widgets = vec![
            emotes.into(),
            Row::with_children(vec![
                TextInput::new(
                    &mut self.new_emote_name_state,
                    "Enter emote name...",
                    &self.new_emote_name,
                    Message::NewEmoteNameChanged,
                )
                .style(theme)
                .width(length!(%2))
                .padding(PADDING / 2)
                .into(),
                space!(w % 1).into(),
                label_button!(&mut self.new_emote_upload_state, "Upload emote")
                    .on_press(Message::UploadEmote)
                    .style(theme)
                    .into(),
            ])
            .spacing(SPACING)
            .align_items(Align::Center)
            .into(),
        ];

        Container::new(
            Card::new(
                label!(format!(
                    "Manage emotes for {}",
                    client
                        .emote_packs
                        .get(&self.pack_id)
                        .map_or("unknown", |pack| pack.pack_name.as_str())
                ))
                .width(length!(= 600 - PADDING - SPACING - (PADDING / 2))),
                column(widgets).width(length!(= 600)).height(length!(= 600)),
            )
            .style(theme)
            .on_close(Message::GoBack),
        )
        .style(theme.border_width(0.0))
        .center_x()
        .center_y()
        .into()
    }

    pub fn update(&mut self, message: Message, client: &Client) -> (Command<TopLevelMessage>, bool) {
        (
            match message {
                Message::GoBack => return (Command::none(), true),
                Message::NewEmoteNameChanged(name) => {
                    self.new_emote_name = name;
                    Command::none()
                }
                Message::UploadEmote => {
                    let pack_id = self.pack_id;
                    let name = self.new_emote_name.drain(..).collect::<String>();
                    let content_store = client.content_store_arc();
                    client.mk_cmd(
                        |inner| async move {
                            let mut emote_file = select_upload_files(&inner, content_store, true).await?;
                            let image_id = emote_file.pop().unwrap().id;
                            inner
                                .call(AddEmoteToPack::new(pack_id, Emote::new(image_id.into(), name)))
                                .await
                                .map_err(ClientError::from)
                        },
                        map_to_nothing,
                    )
                }
                Message::DeleteEmote(name) => {
                    let pack_id = self.pack_id;
                    client.mk_cmd(
                        |inner| async move { inner.call(DeleteEmoteFromPack::new(pack_id, name)).await },
                        map_to_nothing,
                    )
                }
                Message::CopyToClipboard(value) => iced::clipboard::write(value),
            },
            false,
        )
    }
}
