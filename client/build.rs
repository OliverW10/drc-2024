use std::io::Result;

fn main() -> Result<()> {
    let all_messages = &[
        "./messages/path.proto",
        "./messages/command.proto",
        "./messages/diagnostic.proto",
    ];
    prost_build::compile_protos(all_messages, &["messages/"])?;
    Ok(())
}
