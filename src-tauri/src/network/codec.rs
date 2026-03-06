use prost::Message;

use crate::error::{AppError, AppResult};
use crate::proto::gatepb;

/// Service routing info for protobuf messages
pub struct ServiceRoute {
    pub service: &'static str,
    pub method: &'static str,
}

// User service
pub const LOGIN: ServiceRoute = ServiceRoute {
    service: "gamepb.userpb.UserService",
    method: "Login",
};
pub const HEARTBEAT: ServiceRoute = ServiceRoute {
    service: "gamepb.userpb.UserService",
    method: "Heartbeat",
};
pub const REPORT_ARK_CLICK: ServiceRoute = ServiceRoute {
    service: "gamepb.userpb.UserService",
    method: "ReportArkClick",
};

// Plant service
pub const ALL_LANDS: ServiceRoute = ServiceRoute {
    service: "gamepb.plantpb.PlantService",
    method: "AllLands",
};
pub const HARVEST: ServiceRoute = ServiceRoute {
    service: "gamepb.plantpb.PlantService",
    method: "Harvest",
};
pub const WATER_LAND: ServiceRoute = ServiceRoute {
    service: "gamepb.plantpb.PlantService",
    method: "WaterLand",
};
pub const WEED_OUT: ServiceRoute = ServiceRoute {
    service: "gamepb.plantpb.PlantService",
    method: "WeedOut",
};
pub const INSECTICIDE: ServiceRoute = ServiceRoute {
    service: "gamepb.plantpb.PlantService",
    method: "Insecticide",
};
pub const PLANT: ServiceRoute = ServiceRoute {
    service: "gamepb.plantpb.PlantService",
    method: "Plant",
};
pub const REMOVE_PLANT: ServiceRoute = ServiceRoute {
    service: "gamepb.plantpb.PlantService",
    method: "RemovePlant",
};
pub const FERTILIZE: ServiceRoute = ServiceRoute {
    service: "gamepb.plantpb.PlantService",
    method: "Fertilize",
};
pub const PUT_INSECTS: ServiceRoute = ServiceRoute {
    service: "gamepb.plantpb.PlantService",
    method: "PutInsects",
};
pub const PUT_WEEDS: ServiceRoute = ServiceRoute {
    service: "gamepb.plantpb.PlantService",
    method: "PutWeeds",
};
pub const UPGRADE_LAND: ServiceRoute = ServiceRoute {
    service: "gamepb.plantpb.PlantService",
    method: "UpgradeLand",
};
pub const UNLOCK_LAND: ServiceRoute = ServiceRoute {
    service: "gamepb.plantpb.PlantService",
    method: "UnlockLand",
};
pub const CHECK_CAN_OPERATE: ServiceRoute = ServiceRoute {
    service: "gamepb.plantpb.PlantService",
    method: "CheckCanOperate",
};

// Friend service
pub const GET_ALL_FRIENDS: ServiceRoute = ServiceRoute {
    service: "gamepb.friendpb.FriendService",
    method: "GetAll",
};
pub const SYNC_ALL_FRIENDS: ServiceRoute = ServiceRoute {
    service: "gamepb.friendpb.FriendService",
    method: "SyncAll",
};
pub const GET_APPLICATIONS: ServiceRoute = ServiceRoute {
    service: "gamepb.friendpb.FriendService",
    method: "GetApplications",
};
pub const ACCEPT_FRIENDS: ServiceRoute = ServiceRoute {
    service: "gamepb.friendpb.FriendService",
    method: "AcceptFriends",
};
pub const REJECT_FRIENDS: ServiceRoute = ServiceRoute {
    service: "gamepb.friendpb.FriendService",
    method: "RejectFriends",
};

// Visit service
pub const VISIT_ENTER: ServiceRoute = ServiceRoute {
    service: "gamepb.visitpb.VisitService",
    method: "Enter",
};
pub const VISIT_LEAVE: ServiceRoute = ServiceRoute {
    service: "gamepb.visitpb.VisitService",
    method: "Leave",
};

// Item service
pub const BAG: ServiceRoute = ServiceRoute {
    service: "gamepb.itempb.ItemService",
    method: "Bag",
};
pub const SELL: ServiceRoute = ServiceRoute {
    service: "gamepb.itempb.ItemService",
    method: "Sell",
};
pub const USE_ITEM: ServiceRoute = ServiceRoute {
    service: "gamepb.itempb.ItemService",
    method: "Use",
};
pub const BATCH_USE: ServiceRoute = ServiceRoute {
    service: "gamepb.itempb.ItemService",
    method: "BatchUse",
};

// Task service
pub const TASK_INFO: ServiceRoute = ServiceRoute {
    service: "gamepb.taskpb.TaskService",
    method: "TaskInfo",
};
pub const CLAIM_TASK_REWARD: ServiceRoute = ServiceRoute {
    service: "gamepb.taskpb.TaskService",
    method: "ClaimTaskReward",
};
pub const BATCH_CLAIM_TASK_REWARD: ServiceRoute = ServiceRoute {
    service: "gamepb.taskpb.TaskService",
    method: "BatchClaimTaskReward",
};
pub const CLAIM_DAILY_REWARD: ServiceRoute = ServiceRoute {
    service: "gamepb.taskpb.TaskService",
    method: "ClaimDailyReward",
};

// Email service
pub const GET_EMAIL_LIST: ServiceRoute = ServiceRoute {
    service: "gamepb.emailpb.EmailService",
    method: "GetEmailList",
};
pub const CLAIM_EMAIL: ServiceRoute = ServiceRoute {
    service: "gamepb.emailpb.EmailService",
    method: "ClaimEmail",
};
pub const BATCH_CLAIM_EMAIL: ServiceRoute = ServiceRoute {
    service: "gamepb.emailpb.EmailService",
    method: "BatchClaimEmail",
};

// Shop service
pub const SHOP_PROFILES: ServiceRoute = ServiceRoute {
    service: "gamepb.shoppb.ShopService",
    method: "ShopProfiles",
};
pub const SHOP_INFO: ServiceRoute = ServiceRoute {
    service: "gamepb.shoppb.ShopService",
    method: "ShopInfo",
};
pub const BUY_GOODS: ServiceRoute = ServiceRoute {
    service: "gamepb.shoppb.ShopService",
    method: "BuyGoods",
};

// Mall service
pub const GET_MALL_LIST: ServiceRoute = ServiceRoute {
    service: "gamepb.mallpb.MallService",
    method: "GetMallListBySlotType",
};
pub const PURCHASE: ServiceRoute = ServiceRoute {
    service: "gamepb.mallpb.MallService",
    method: "Purchase",
};
pub const GET_MONTH_CARD_INFOS: ServiceRoute = ServiceRoute {
    service: "gamepb.mallpb.MallService",
    method: "GetMonthCardInfos",
};
pub const CLAIM_MONTH_CARD_REWARD: ServiceRoute = ServiceRoute {
    service: "gamepb.mallpb.MallService",
    method: "ClaimMonthCardReward",
};

// QQ VIP service
pub const GET_DAILY_GIFT_STATUS: ServiceRoute = ServiceRoute {
    service: "gamepb.qqvippb.QQVipService",
    method: "GetDailyGiftStatus",
};
pub const CLAIM_DAILY_GIFT: ServiceRoute = ServiceRoute {
    service: "gamepb.qqvippb.QQVipService",
    method: "ClaimDailyGift",
};

// Red packet service
pub const GET_TODAY_CLAIM_STATUS: ServiceRoute = ServiceRoute {
    service: "gamepb.redpacketpb.RedPacketService",
    method: "GetTodayClaimStatus",
};
pub const CLAIM_RED_PACKET: ServiceRoute = ServiceRoute {
    service: "gamepb.redpacketpb.RedPacketService",
    method: "ClaimRedPacket",
};

// Illustrated service
pub const GET_ILLUSTRATED_LIST: ServiceRoute = ServiceRoute {
    service: "gamepb.illustratedpb.IllustratedService",
    method: "GetIllustratedListV2",
};
pub const CLAIM_ALL_REWARDS_V2: ServiceRoute = ServiceRoute {
    service: "gamepb.illustratedpb.IllustratedService",
    method: "ClaimAllRewardsV2",
};

// Share service
pub const CHECK_CAN_SHARE: ServiceRoute = ServiceRoute {
    service: "gamepb.sharepb.ShareService",
    method: "CheckCanShare",
};
pub const REPORT_SHARE: ServiceRoute = ServiceRoute {
    service: "gamepb.sharepb.ShareService",
    method: "ReportShare",
};
pub const CLAIM_SHARE_REWARD: ServiceRoute = ServiceRoute {
    service: "gamepb.sharepb.ShareService",
    method: "ClaimShareReward",
};

/// Encode a protobuf request into a gateway message frame
pub fn encode_gate_message(
    route: &ServiceRoute,
    body: Vec<u8>,
    client_seq: i64,
    server_seq: i64,
) -> Vec<u8> {
    let msg = gatepb::Message {
        meta: Some(gatepb::Meta {
            service_name: route.service.to_string(),
            method_name: route.method.to_string(),
            message_type: gatepb::MessageType::Request as i32,
            client_seq,
            server_seq,
            error_code: 0,
            error_message: String::new(),
            metadata: Default::default(),
        }),
        body,
    };
    msg.encode_to_vec()
}

/// Decode a gateway message frame
pub fn decode_gate_message(data: &[u8]) -> AppResult<gatepb::Message> {
    gatepb::Message::decode(data).map_err(|e| AppError::Protocol(e.to_string()))
}
