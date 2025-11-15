use arb_bot::logger::{info, LoggerConfig, LogFormat};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    
    // Initialize logger
    LoggerConfig::new()
        .with_level("info")
        .with_format(LogFormat::Pretty)
        .init()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to initialize logger: {}", e))?;
    
    info!("Hello, world!");
    Ok(())
}
