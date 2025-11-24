use ai_os::smartfs::SmartFs;
use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    env_logger::init();

    // Usage:
    // sudo ./target/debug/ai_os <real_dir> <mount_dir>
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <real_dir> <mount_dir>", args[0]);
        std::process::exit(1);
    }

    let real_dir = PathBuf::from(&args[1]);
    let mount_dir = PathBuf::from(&args[2]);

    let fs = SmartFs::new(real_dir)?;

    // FUSE options basiques
    let options = [
        fuser::MountOption::RW,
        fuser::MountOption::FSName("ai_os_smartfs".to_string()),
        fuser::MountOption::AutoUnmount,
        fuser::MountOption::DefaultPermissions,
    ];

    // Bloquant tant que monté
    fuser::mount2(fs, &mount_dir, &options)?;

    Ok(())
}
