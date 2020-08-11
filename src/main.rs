use main_error::MainError;
use ryzen_reader::CpuInfo;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let mut env: HashMap<String, String> = dotenv::vars().collect();
    let host = env
        .remove("HOSTNAME")
        .or_else(|| {
            hostname::get()
                .ok()
                .and_then(|hostname| hostname.into_string().ok())
        })
        .ok_or("No HOSTNAME set or detected")?;
    let port = env
        .get("PORT")
        .and_then(|s| u16::from_str(s).ok())
        .unwrap_or(80);

    let cpu_info = Arc::new(CpuInfo::new()?);

    let metrics = warp::path!("metrics").map(move || {
        let power = match cpu_info.read() {
            Ok(power) => power,
            Err(e) => {
                eprintln!("{}", e);
                return String::new();
            }
        };

        let package_lines = power.packages().enumerate().map(|(package, power)| {
            format!(
                "package_power{{package=\"{}\", host=\"{}\"}} {}",
                package, host, power
            )
        });
        let core_lines = power.cores().enumerate().map(|(package, power)| {
            format!(
                "core_power{{package=\"{}\", host=\"{}\"}} {}",
                package, host, power
            )
        });
        let lines: Vec<_> = package_lines.chain(core_lines).collect();
        lines.join("\n")
    });

    ctrlc::set_handler(move || {
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    warp::serve(metrics).run(([0, 0, 0, 0], port)).await;

    Ok(())
}
