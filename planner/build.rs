use std::io::Result;

fn main() -> Result<()> {
    let project_root_dir = std::env::current_dir().expect("msg").into_os_string();
    let messages_dir = project_root_dir.push("messages")
    println!("{}", project_root_dir);
    prost_build::compile_protos(&[project_root_dir + "messages/*.proto"], &[project_root_dir"messages"])?;
    Ok(())
}
