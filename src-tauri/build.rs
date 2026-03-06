fn main() {
    prost_build::Config::new()
        .out_dir("src/proto")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &[
                "proto/game.proto",
                "proto/corepb.proto",
                "proto/plantpb.proto",
                "proto/userpb.proto",
                "proto/friendpb.proto",
                "proto/taskpb.proto",
                "proto/itempb.proto",
                "proto/visitpb.proto",
                "proto/shoppb.proto",
                "proto/emailpb.proto",
                "proto/mallpb.proto",
                "proto/redpacketpb.proto",
                "proto/qqvippb.proto",
                "proto/illustratedpb.proto",
                "proto/sharepb.proto",
                "proto/notifypb.proto",
            ],
            &["proto/"],
        )
        .expect("Failed to compile proto files");

    tauri_build::build()
}
