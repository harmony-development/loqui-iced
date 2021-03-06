use crate::{
    client::{
        channel::Channels,
        guild::{Guild, Guilds},
    },
    component::*,
    label,
    screen::{main::Message, truncate_string},
    space,
    style::{Theme, AVATAR_WIDTH, DEF_SIZE, PADDING, SPACING},
};

use client::{bool_ext::BoolExt, channel::Channel};
use iced::{tooltip::Position, Tooltip};

/// Builds a room list.
#[allow(clippy::too_many_arguments)]
pub fn build_channel_list<'a>(
    channels: &Channels,
    current_channel_id: Option<u64>,
    state: &'a mut scrollable::State,
    buttons_state: &'a mut [button::State],
    on_button_press: fn(u64) -> Message,
    theme: &Theme,
) -> Element<'a, Message> {
    type Item<'a, 'b> = ((&'b u64, &'b Channel), &'a mut button::State);
    let process_item = |mut list: Scrollable<'a, Message>, ((channel_id, channel), button_state): Item<'a, '_>| {
        let read_color = channel.has_unread.then(|| theme.user_theme.text).unwrap_or(Color {
            r: theme.user_theme.dimmed_text.r * 0.7,
            g: theme.user_theme.dimmed_text.g * 0.7,
            b: theme.user_theme.dimmed_text.b * 0.7,
            a: theme.user_theme.dimmed_text.a,
        });

        let mut content_widgets = Vec::with_capacity(5);
        content_widgets.push(channel_icon(channel));
        channel
            .is_category
            .and_do(|| content_widgets.push(space!(w = SPACING).into()));
        content_widgets.push(label!(truncate_string(&channel.name, 17)).size(DEF_SIZE - 2).into());

        let mut but = Button::new(
            button_state,
            Row::with_children(content_widgets).align_items(Align::Center),
        )
        .width(length!(+))
        .style(theme.secondary().text_color(read_color));

        if channel.is_category {
            but = but.style(theme.embed().border_width(0.0).text_color(read_color));
        } else if current_channel_id != Some(*channel_id) {
            but = but.on_press(on_button_press(*channel_id));
        }

        list = list.push(but);

        if channel.is_category {
            list = list.push(Rule::horizontal(SPACING).style(theme.secondary()));
        }

        list
    };

    let list_init = Scrollable::new(state)
        .style(theme)
        .align_items(Align::Start)
        .height(length!(+))
        .spacing(SPACING)
        .padding(PADDING / 4);

    channels
        .iter()
        .zip(buttons_state.iter_mut())
        .fold(list_init, process_item)
        .into()
}

#[allow(clippy::too_many_arguments)]
pub fn build_guild_list<'a>(
    guilds: &Guilds,
    thumbnail_cache: &ThumbnailCache,
    current_guild_id: Option<u64>,
    state: &'a mut scrollable::State,
    buttons_state: &'a mut [button::State],
    on_button_press: fn(u64) -> Message,
    theme: &Theme,
) -> Element<'a, Message> {
    let buttons_state_len = buttons_state.len();

    type Item<'a, 'b> = ((&'b u64, &'b Guild), (usize, &'a mut button::State));
    let process_item = |mut list: Scrollable<'a, Message>, ((guild_id, guild), (index, button_state)): Item<'a, '_>| {
        let mk_but = |state: &'a mut button::State, content: Element<'a, Message>| {
            let theme = if guild.channels.values().any(|c| c.has_unread) {
                theme.border_color(Color::WHITE)
            } else {
                *theme
            };

            Button::new(state, fill_container(content).style(theme.border_width(0.0)))
                .width(length!(+))
                .height(length!(= 52))
                .style(theme.secondary().border_width(2.0))
        };

        let but = if index >= buttons_state_len - 1 {
            // [ref:create_join_guild_but_state]
            mk_but(button_state, icon(Icon::Plus).size(DEF_SIZE + 10).into()).on_press(Message::OpenCreateJoinGuild)
        } else {
            let content = guild
                .picture
                .as_ref()
                .and_then(|guild_picture| {
                    thumbnail_cache
                        .avatars
                        .get(guild_picture)
                        .or_else(|| thumbnail_cache.thumbnails.get(guild_picture))
                })
                .map_or_else::<Element<Message>, _, _>(
                    || {
                        label!(guild.name.chars().next().unwrap_or('u').to_ascii_uppercase())
                            .size(DEF_SIZE + 10)
                            .into()
                    },
                    |handle| {
                        Image::new(handle.clone())
                            .width(length!(= AVATAR_WIDTH - 4))
                            .height(length!(= AVATAR_WIDTH - 4))
                            .into()
                    },
                );

            let mut but = mk_but(button_state, content);

            if current_guild_id != Some(*guild_id) {
                but = but.on_press(on_button_press(*guild_id));
            }

            but
        };

        list = list.push(
            Tooltip::new(but, &guild.name, Position::Bottom)
                .gap(PADDING / 2)
                .style(theme.secondary()),
        );

        if index < buttons_state_len - 1 {
            list = list.push(Rule::horizontal(SPACING).style(theme.secondary()));
        }

        list
    };
    let list_init = Scrollable::new(state)
        .style(theme)
        .align_items(Align::Start)
        .height(length!(+))
        .spacing(SPACING)
        .padding(PADDING / 4);

    guilds
        .into_iter()
        .chain(std::iter::once((
            &0,
            &Guild {
                name: String::from("Create / join guild"),
                ..Default::default()
            },
        ))) // [ref:create_join_guild_but_state]
        .zip(buttons_state.iter_mut().enumerate())
        .fold(list_init, process_item)
        .into()
}
