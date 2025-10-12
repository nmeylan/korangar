use std::net::IpAddr;
use std::time::Instant;
use ragnarok_packets::handler::{DuplicateHandlerError, PacketCallback, PacketHandler};
use ragnarok_packets::{BuyOrSellPacket, ChangeMapPacket, CharacterCreationFailedPacket, CharacterCreationFailedReason, CharacterDeletionFailedPacket, CharacterDeletionFailedReason, CharacterDeletionSuccessPacket, CharacterInformation, CharacterListPacket_20100803, CharacterSelectionFailedPacket, CharacterSelectionFailedReason, CharacterSelectionSuccessPacket, CloseButtonPacket, CreateCharacterSuccessPacket, DialogMenuPacket, DisconnectResponsePacket, DisconnectResponseStatus, EntityMovePacket, EntityStopMovePacket, HotkeyData, InitialStatsPacket, LoginBannedPacked, LoginFailedPacket2, LoginFailedReason, LoginFailedReason2, LoginPincodePacket, LoginServerLoginSuccessPacket, MapServerLoginSuccessPacket, MapServerPingPacket, MapServerUnavailablePacket, MapTypePacket, NextButtonPacket, NpcDialogPacket, Packet0b18, Packet8302, PlayerMovePacket, RestartResponsePacket, RestartResponseStatus, SpriteChangePacket, SpriteChangeType, SwitchCharacterSlotResponsePacket, SwitchCharacterSlotResponseStatus, UpdateHotkeysPacket, UpdateSkillTreePacket, UpdateStatPacket, UpdateStatPacket1, UpdateStatPacket2, UpdateStatPacket3};

use crate::event::{NetworkEventList, NoNetworkEvents};
use crate::{CharacterServerLoginData, HotkeyState, LoginServerLoginData, MessageColor, NetworkEvent, UnifiedCharacterSelectionFailedReason, UnifiedLoginFailedReason};

pub fn register_login_server_packets<Callback>(
    packet_handler: &mut PacketHandler<NetworkEventList, (), Callback>,
) -> Result<(), DuplicateHandlerError>
where
    Callback: PacketCallback,
{
    packet_handler.register(|packet: LoginServerLoginSuccessPacket| {
        NetworkEvent::LoginServerConnected {
            character_servers: packet.character_server_information,
            login_data: LoginServerLoginData {
                account_id: packet.account_id,
                login_id1: packet.login_id1 as u32,
                login_id2: packet.login_id2,
                sex: packet.sex,
            },
        }
    })?;
    packet_handler.register(|packet: LoginBannedPacked| {
        let (reason, message) = match packet.reason {
            LoginFailedReason::ServerClosed => (UnifiedLoginFailedReason::ServerClosed, "Server closed"),
            LoginFailedReason::AlreadyLoggedIn => (
                UnifiedLoginFailedReason::AlreadyLoggedIn,
                "Someone has already logged in with this id",
            ),
            LoginFailedReason::AlreadyOnline => (UnifiedLoginFailedReason::AlreadyOnline, "Already online"),
        };

        NetworkEvent::LoginServerConnectionFailed { reason, message }
    })?;
    packet_handler.register(|packet: LoginFailedPacket2| {
        let (reason, message) = match packet.reason {
            LoginFailedReason2::UnregisteredId => (UnifiedLoginFailedReason::UnregisteredId, "Unregistered id"),
            LoginFailedReason2::IncorrectPassword => (UnifiedLoginFailedReason::IncorrectPassword, "Incorrect password"),
            LoginFailedReason2::IdExpired => (UnifiedLoginFailedReason::IdExpired, "Id has expired"),
            LoginFailedReason2::RejectedFromServer => (UnifiedLoginFailedReason::RejectedFromServer, "Rejected from server"),
            LoginFailedReason2::BlockedByGMTeam => (UnifiedLoginFailedReason::BlockedByGMTeam, "Blocked by gm team"),
            LoginFailedReason2::GameOutdated => (UnifiedLoginFailedReason::GameOutdated, "Game outdated"),
            LoginFailedReason2::LoginProhibitedUntil => (UnifiedLoginFailedReason::LoginProhibitedUntil, "Login prohibited until"),
            LoginFailedReason2::ServerFull => (UnifiedLoginFailedReason::ServerFull, "Server is full"),
            LoginFailedReason2::CompanyAccountLimitReached => (
                UnifiedLoginFailedReason::CompanyAccountLimitReached,
                "Company account limit reached",
            ),
        };

        NetworkEvent::LoginServerConnectionFailed { reason, message }
    })?;

    Ok(())
}

pub fn register_character_server_packets<Callback>(
    packet_handler: &mut PacketHandler<NetworkEventList, (), Callback>,
) -> Result<(), DuplicateHandlerError>
where
    Callback: PacketCallback,
{
    packet_handler.register(|packet: LoginBannedPacked| {
        let reason = packet.reason;
        let message = match reason {
            LoginFailedReason::ServerClosed => "Server closed",
            LoginFailedReason::AlreadyLoggedIn => "Someone has already logged in with this id",
            LoginFailedReason::AlreadyOnline => "Already online",
        };

        NetworkEvent::CharacterServerConnectionFailed { reason, message }
    })?;

    packet_handler.register(|packet: CharacterListPacket_20100803| {
        NetworkEventList::from(vec![
            NetworkEvent::CharacterServerConnected {
                normal_slot_count: packet.maximum_slot_count as usize,
            },
            NetworkEvent::CharacterList {
                characters: packet.character_information.into_iter().map(|c| c.into()).collect::<Vec<CharacterInformation>>(),
            },
        ])

    })?;
    packet_handler.register_noop::<LoginPincodePacket>()?;
    packet_handler.register(|packet: CharacterSelectionSuccessPacket| {
        let login_data = CharacterServerLoginData {
            server_ip: IpAddr::V4(packet.map_server_ip.into()),
            server_port: packet.map_server_port,
            character_id: packet.character_id,
        };

        NetworkEvent::CharacterSelected { login_data }
    })?;
    packet_handler.register(|packet: CharacterSelectionFailedPacket| {
        let (reason, message) = match packet.reason {
            CharacterSelectionFailedReason::RejectedFromServer => (
                UnifiedCharacterSelectionFailedReason::RejectedFromServer,
                "Rejected from server",
            ),
        };

        NetworkEvent::CharacterSelectionFailed { reason, message }
    })?;
    packet_handler.register(|_: MapServerUnavailablePacket| {
        let reason = UnifiedCharacterSelectionFailedReason::MapServerUnavailable;
        let message = "Map server currently unavailable";

        NetworkEvent::CharacterSelectionFailed { reason, message }
    })?;
    packet_handler.register(|packet: CreateCharacterSuccessPacket| NetworkEvent::CharacterCreated {
        character_information: packet.character_information.into(),
    })?;
    packet_handler.register(|packet: CharacterCreationFailedPacket| {
        let reason = packet.reason;
        let message = match reason {
            CharacterCreationFailedReason::CharacterNameAlreadyUsed => "Character name is already used",
            CharacterCreationFailedReason::NotOldEnough => "You are not old enough to create a character",
            CharacterCreationFailedReason::NotAllowedToUseSlot => "You are not allowed to use this character slot",
            CharacterCreationFailedReason::CharacterCerationFailed => "Character creation failed",
        };

        NetworkEvent::CharacterCreationFailed { reason, message }
    })?;
    packet_handler.register(|_: CharacterDeletionSuccessPacket| NetworkEvent::CharacterDeleted)?;
    packet_handler.register(|packet: CharacterDeletionFailedPacket| {
        let reason = packet.reason;
        let message = match reason {
            CharacterDeletionFailedReason::NotAllowed => "You are not allowed to delete this character",
            CharacterDeletionFailedReason::CharacterNotFound => "Character was not found",
            CharacterDeletionFailedReason::NotEligible => "Character is not eligible for deletion",
        };
        NetworkEvent::CharacterDeletionFailed { reason, message }
    })?;
    packet_handler.register(|packet: SwitchCharacterSlotResponsePacket| match packet.status {
        SwitchCharacterSlotResponseStatus::Success => NetworkEvent::CharacterSlotSwitched,
        SwitchCharacterSlotResponseStatus::Error => NetworkEvent::CharacterSlotSwitchFailed,
    })?;

    Ok(())
}

pub fn register_map_server_packets<Callback>(
    packet_handler: &mut PacketHandler<NetworkEventList, (), Callback>,
) -> Result<(), DuplicateHandlerError>
where
    Callback: PacketCallback,
{
    packet_handler.register(|_: MapServerPingPacket| NoNetworkEvents)?;
    packet_handler.register(|packet: MapServerLoginSuccessPacket| NetworkEvent::UpdateClientTick {
        client_tick: packet.client_tick,
        received_at: Instant::now(),
    })?;

    // Map events
    packet_handler.register(|packet: ChangeMapPacket| {
        let ChangeMapPacket { map_name, position } = packet;

        let map_name = map_name.replace(".gat", "");

        NetworkEvent::ChangeMap { map_name, position }
    })?;
    packet_handler.register_noop::<MapTypePacket>()?;
    // UI
    packet_handler.register(|packet: RestartResponsePacket| match packet.result {
        RestartResponseStatus::Ok => NetworkEvent::LoggedOut,
        RestartResponseStatus::Nothing => NetworkEvent::ChatMessage {
            text: "Failed to log out.".to_string(),
            color: MessageColor::Error,
        },
    })?;
    packet_handler.register(|packet: DisconnectResponsePacket| match packet.result {
        DisconnectResponseStatus::Ok => NetworkEvent::LoggedOut,
        DisconnectResponseStatus::Wait10Seconds => NetworkEvent::ChatMessage {
            text: "Please wait 10 seconds before trying to log out.".to_string(),
            color: MessageColor::Error,
        },
    })?;
    packet_handler.register(|packet: UpdateHotkeysPacket| NetworkEvent::SetHotkeyData {
        tab: packet.tab,
        hotkeys: packet
            .hotkeys
            .into_iter()
            .map(|hotkey_data| match hotkey_data == HotkeyData::UNBOUND {
                true => HotkeyState::Unbound,
                false => HotkeyState::Bound(hotkey_data),
            })
            .collect(),
    })?;
    // Movement
    packet_handler.register(|packet: PlayerMovePacket| {
        let PlayerMovePacket {
            starting_timestamp,
            from_to,
        } = packet;

        let (origin, destination) = from_to.to_origin_destination();

        NetworkEvent::PlayerMove {
            origin,
            destination,
            starting_timestamp,
        }
    })?;

    packet_handler.register(|packet: EntityMovePacket| {
        let EntityMovePacket {
            entity_id,
            from_to,
            starting_timestamp,
        } = packet;

        let (origin, destination) = from_to.to_origin_destination();

        NetworkEvent::EntityMove {
            entity_id,
            origin,
            destination,
            starting_timestamp,
        }
    })?;
    packet_handler.register_noop::<EntityStopMovePacket>()?;
    // Stats
    packet_handler.register(|packet: InitialStatsPacket| {
        let InitialStatsPacket {
            strength_stat_points_cost,
            agility_stat_points_cost,
            vitality_stat_points_cost,
            intelligence_stat_points_cost,
            dexterity_stat_points_cost,
            luck_stat_points_cost,
            ..
        } = packet;

        NetworkEvent::InitialStats {
            strength_stat_points_cost,
            agility_stat_points_cost,
            vitality_stat_points_cost,
            intelligence_stat_points_cost,
            dexterity_stat_points_cost,
            luck_stat_points_cost,
        }
    })?;
    packet_handler.register(|packet: UpdateSkillTreePacket| {
        let UpdateSkillTreePacket { skill_information } = packet;
        NetworkEvent::SkillTree { skill_information }
    })?;
    packet_handler.register(|packet: UpdateStatPacket| {
        let UpdateStatPacket { stat_type } = packet;
        NetworkEvent::UpdateStat { stat_type }
    })?;
    packet_handler.register(|packet: UpdateStatPacket1| {
        let UpdateStatPacket1 { stat_type } = packet;
        NetworkEvent::UpdateStat { stat_type }
    })?;
    packet_handler.register(|packet: UpdateStatPacket2| {
        let UpdateStatPacket2 { stat_type } = packet;
        NetworkEvent::UpdateStat { stat_type }
    })?;
    packet_handler.register(|packet: UpdateStatPacket3| {
        let UpdateStatPacket3 { stat_type } = packet;
        NetworkEvent::UpdateStat { stat_type }
    })?;
    packet_handler.register(|packet: SpriteChangePacket| match packet.sprite_type {
        SpriteChangeType::Base => Some(NetworkEvent::ChangeJob {
            account_id: packet.account_id,
            job_id: packet.value,
        }),
        SpriteChangeType::Hair => Some(NetworkEvent::ChangeHair {
            account_id: packet.account_id,
            hair_id: packet.value,
        }),
        _ => None,
    })?;

    // NPC packet
    packet_handler.register(|packet: NextButtonPacket| {
        let NextButtonPacket { npc_id } = packet;

        NetworkEvent::AddNextButton { npc_id }
    })?;
    packet_handler.register(|packet: CloseButtonPacket| {
        let CloseButtonPacket { npc_id } = packet;

        NetworkEvent::AddCloseButton { npc_id }
    })?;
    packet_handler.register(|packet: DialogMenuPacket| {
        let DialogMenuPacket { npc_id, message } = packet;

        let choices = message.split(':').map(String::from).filter(|text| !text.is_empty()).collect();

        NetworkEvent::AddChoiceButtons { choices, npc_id }
    })?;
    packet_handler.register(|packet: NpcDialogPacket| {
        let NpcDialogPacket { npc_id, text } = packet;

        NetworkEvent::OpenDialog { text, npc_id }
    })?;
    packet_handler.register(|packet: BuyOrSellPacket| NetworkEvent::AskBuyOrSell { shop_id: packet.shop_id })?;
    // Skill

    // Unknown
    packet_handler.register_noop::<Packet8302>()?;
    packet_handler.register_noop::<Packet0b18>()?;
    Ok(())
}
