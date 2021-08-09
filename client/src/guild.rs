use ahash::AHashMap;
use harmony_rust_sdk::client::api::rest::FileId;

use crate::{
    role::{Role, Roles},
    IndexMap,
};

use super::channel::Channels;

pub type Guilds = IndexMap<u64, Guild>;

#[derive(Debug, Clone, Default)]
pub struct Guild {
    pub name: String,
    pub picture: Option<FileId>,
    pub channels: Channels,
    pub roles: Roles,
    pub members: AHashMap<u64, Vec<u64>>,
    pub homeserver: String,
    pub user_perms: GuildPerms,
    pub init_fetching: bool,
}

impl Guild {
    pub fn update_channel_order(&mut self, previous_id: u64, next_id: u64, channel_id: u64) {
        update_order(&mut self.channels, previous_id, next_id, channel_id)
    }

    pub fn update_role_order(&mut self, previous_id: u64, next_id: u64, role_id: u64) {
        update_order(&mut self.roles, previous_id, next_id, role_id)
    }

    pub fn highest_role_for_member(&self, user_id: u64) -> Option<&Role> {
        self.members.get(&user_id).and_then(|role_ids| {
            self.roles
                .iter()
                .find(|(id, _)| role_ids.contains(id))
                .map(|(_, role)| role)
        })
    }
}

fn update_order<V>(map: &mut IndexMap<u64, V>, previous_id: u64, next_id: u64, id: u64) {
    if let Some(item_pos) = map.get_index_of(&id) {
        let prev_pos = map.get_index_of(&previous_id);
        let next_pos = map.get_index_of(&next_id);

        if let Some(pos) = prev_pos {
            let pos = pos + 1;
            if pos != item_pos && pos < map.len() {
                map.swap_indices(pos, item_pos);
            }
        } else if let Some(pos) = next_pos {
            if pos != 0 {
                map.swap_indices(pos - 1, item_pos);
            } else {
                let (k, v) = map.pop().unwrap();
                map.reverse();
                map.insert(k, v);
                map.reverse();
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GuildPerms {
    pub change_info: bool,
}
