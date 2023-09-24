#![warn(clippy::cargo)]

use clap::Parser;
use wifi_qr_code_generator::{EapMethod, GenerationError, ImageFormat, Phase2, Wifi, WifiMethod};

#[derive(Debug, clap::Parser)]
struct CliArgs {
    ssid: String,
    #[arg(value_enum)]
    kind: Option<WifiMethod>,
    #[arg(long = "hidden")]
    hidden: bool,
    #[arg(long = "eap", value_enum)]
    eap_method: Option<EapMethod>,
    #[arg(long = "ph2", value_enum)]
    phase2: Option<Phase2>,
    #[arg(long = "anonymous_identity", short = 'a')]
    anonymous_identity: Option<String>,
    #[arg(long = "identity", short = 'i')]
    identity: Option<String>,
    #[arg(long = "password", short = 'p')]
    password: Option<String>,
    #[arg(long, default_value_t, value_enum)]
    image_format: ImageFormat,
}

fn main() -> Result<(), GenerationError> {
    let args = CliArgs::parse();

    let file_name = if let Some(ident) = &args.identity {
        format!("./wifi-{}-{ident}.png", args.ssid)
    } else {
        format!("./wifi-{}.png", args.ssid)
    };

    let wifi = Wifi::new(args.ssid)
        .with_method(args.kind)
        .with_hidden(args.hidden)
        .with_eap_method(args.eap_method)
        .with_phase2(args.phase2)
        .with_anonymous_identity(args.anonymous_identity)
        .with_identity(args.identity)
        .with_password(args.password);

    let wifi_string = wifi.to_string();
    println!("{}", wifi_string);

    wifi.generate_image_file(Some(args.image_format), file_name.as_ref())?;

    Ok(())
}
