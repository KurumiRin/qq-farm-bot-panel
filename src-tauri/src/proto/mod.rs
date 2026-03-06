pub mod gatepb {
    include!("gatepb.rs");
}

pub mod corepb {
    include!("corepb.rs");
}

/// gamepb.* sub-packages. Nested to match prost's generated `super::` paths.
/// - gamepb.taskpb references `super::super::corepb` -> goes up to `gamepb` then to `proto`
/// - gamepb.visitpb references `super::userpb`, `super::plantpb` -> sibling modules
mod gamepb {
    pub mod plantpb {
        include!("gamepb.plantpb.rs");
    }

    pub mod userpb {
        include!("gamepb.userpb.rs");
    }

    pub mod friendpb {
        include!("gamepb.friendpb.rs");
    }

    pub mod taskpb {
        include!("gamepb.taskpb.rs");
    }

    pub mod itempb {
        include!("gamepb.itempb.rs");
    }

    pub mod visitpb {
        include!("gamepb.visitpb.rs");
    }

    pub mod shoppb {
        include!("gamepb.shoppb.rs");
    }

    pub mod emailpb {
        include!("gamepb.emailpb.rs");
    }

    pub mod mallpb {
        include!("gamepb.mallpb.rs");
    }

    pub mod redpacketpb {
        include!("gamepb.redpacketpb.rs");
    }

    pub mod qqvippb {
        include!("gamepb.qqvippb.rs");
    }

    pub mod illustratedpb {
        include!("gamepb.illustratedpb.rs");
    }

    pub mod sharepb {
        include!("gamepb.sharepb.rs");
    }
}

// Re-export for convenient access: `proto::plantpb`, `proto::userpb`, etc.
pub use gamepb::*;
