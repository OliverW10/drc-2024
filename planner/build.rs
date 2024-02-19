use std::{io::Result};

fn main() -> Result<()> {
    // let path = std::env::current_dir().expect("msg");
    // path.push(PathBuf::from("messages"));
    let all_messages = &[
        "./messages/path.proto",
        "./messages/odometry.proto",
        "./messages/command.proto",
        "./messages/diagnostic.proto",
    ];
    prost_build::compile_protos(all_messages, &["messages/"])?;
    // let messages = ["command", "odometry", "path"];
    // let protos = messages.map(|msg| { path.with_file_name(msg).with_extension(".proto") });
    // // let path_str = path.to_str().expect("msg");
    // println!("{:?}", protos);
    // prost_build::compile_protos(&[protos], &[path_str.to_owned() + "/messages"])?;
    Ok(())
}
