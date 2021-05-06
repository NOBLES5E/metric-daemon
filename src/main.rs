#[macro_use]
extern crate shadow_rs;

use structopt::StructOpt;
use anyhow::Result;
use pretty_assertions::{assert_eq, assert_ne};
use std::time::Duration;
use std::process::Stdio;

#[derive(Debug, StructOpt, Clone)]
#[structopt()]
struct Cli {
    #[structopt(long, env = "METRIC_NAME")]
    name: String,
    #[structopt(long, env = "METRIC_PROPERTIES")]
    properties: Option<String>,
    #[structopt(long, env = "METRIC_SERVER_URL")]
    server_url: String,
    #[structopt(long, env = "METRIC_COMMAND")]
    command: String,
    #[structopt(long, default_value = "10")]
    interval_seconds: usize,
}

fn main() -> Result<()> {
    color_eyre::install().unwrap();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("LOG_LEVEL"))
        .init();
    shadow!(build);
    eprintln!("commit_hash: {}", build::COMMIT_HASH);
    let args: Cli = Cli::from_args();

    loop {
        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c").arg(args.command.as_str());
        cmd.stdout(Stdio::piped()).stderr(Stdio::inherit());
        let child = cmd.spawn()?;
        let output = child.wait_with_output()?;
        assert!(output.status.success());
        let value: f32 = String::from_utf8_lossy(output.stdout.as_ref()).trim().parse()?;
        let status = match args.properties.as_ref() {
            None => {
                ureq::post(args.server_url.as_str())
                    .send_string(format!("{},hostname={} value={}", args.name, hostname::get()?.into_string().expect("cannot get hostname"), value).as_str())?
                    .status()
            }
            Some(properties) => {
                ureq::post(args.server_url.as_str())
                    .send_string(format!("{},{},hostname={} value={}", args.name, properties, hostname::get()?.into_string().expect("cannot get hostname"), value).as_str())?
                    .status()
            }
        };
        assert!(200 <= status && status < 300);
        std::thread::sleep(Duration::from_secs(args.interval_seconds as _));
    }
}
