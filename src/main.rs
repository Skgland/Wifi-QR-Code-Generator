#![warn(clippy::cargo)]

use clap::Parser;
use image::Luma;
use qrcode::QrCode;

fn main() {
    let wifi = Wifi::parse();
    let wifi_string =wifi.to_string();
    println!("{}", wifi_string);
    let code = QrCode::new(wifi_string).unwrap();
    let image = code.render::<Luma<u8>>().build();

    let file_name = if let Some(ident) = wifi.identity {
        format!("./wifi-{}-{ident}.png", wifi.ssid)
    } else {
        format!("./wifi-{}.png", wifi.ssid)
    }    ;

    image.save(file_name).unwrap();
}

impl ToString for Wifi {

    fn to_string(&self) -> String {

        //TODO all fields should have contained ; and \ backslash escaped

        let kind = match self.kind {
            WifiMethod::NoPass => "nopass",
            WifiMethod::Wep => "WEP",
            WifiMethod::Wpa => "WPA",
            WifiMethod::Wpa2 => "WPA2",
            WifiMethod::Wpa2Enterprise => "WPA2-EAP",
            WifiMethod::Wpa3 => "WPA3",
        };

        let ssid = escape_field_value(&self.ssid);
        let hidden = if self.hidden {"H:true;"} else {""};
        let eap = if let Some(eap) = &self.eap_method {
            let eap_name = match eap {
                EapMethod::Peap => "PEAP",
                EapMethod::Tls => "TLS",
                EapMethod::Ttls => "TTLS",
                EapMethod::Pwd => "PWD",
                EapMethod::Sim => "SIM",
                EapMethod::Aka => "AKA",
                EapMethod::AkaPrime => "AKA_PRIME",
            };
            format!("E:{eap_name};")
        } else {
            String::new()
        };
        let phase2 = if let Some(ph2) = &self.phase2  {
            let ph2_name = match ph2 {
                Phase2::MsChap => "MSCHAP",
                Phase2::MsChapV2 => "MSCHAPV2",
                Phase2::Gtc => "GTC",
                Phase2::Sim => "SIM",
                Phase2::Aka => "AKA",
                Phase2::AkaPrime => "AKA_PRIME",
                Phase2::Pap => "PAP",
            };
            format!("PH2:{ph2_name};")
        } else {
            String::new()
        };
        let anon = if let Some(anon) = &self.anonymous_identity {
            let anon = escape_field_value(anon);
            format!("A:{anon};")
        } else {
            String::new()
        };
        let ident = if let Some(ident) = &self.identity {
            let ident = escape_field_value(ident);
            format!("I:{ident};")
        } else {
            String::new()
        };
        let password = if let Some(password) = &self.password {
            let password = escape_field_value(password);
            format!("P:{password};")
        } else {
            String::new()
        };

        format!("WIFI:T:{kind};S:{ssid};{hidden}{eap}{phase2}{anon}{ident}{password};;")
    }
}

#[derive(Debug, clap::Parser)]
struct Wifi {
    ssid: String,
    kind: WifiMethod,
    #[arg(long = "hidden")]
    hidden: bool,
    #[arg(long = "password", short = 'p')]
    password: Option<String>,
    #[arg(long = "eap")]
    eap_method: Option<EapMethod>,
    #[arg(long = "ph2")]
    phase2: Option<Phase2>,
    #[arg(long = "identity", short = 'i')]
    identity: Option<String>,
    #[arg(long = "anonymous_identity", short = 'a')]
    anonymous_identity: Option<String>,
}

#[derive(Debug,Clone,clap::ValueEnum)]
enum WifiMethod
{
    NoPass,
    Wep,
    Wpa,
    Wpa2,
    Wpa2Enterprise,
    Wpa3
}

#[derive(Debug,Clone,clap::ValueEnum)]
enum EapMethod {
    Peap,
    Tls,
    Ttls,
    Pwd,
    Sim,
    Aka,
    AkaPrime,
}

#[derive(Debug,Clone,clap::ValueEnum)]
enum Phase2 {
    MsChap,
    MsChapV2,
    Pap,
    Gtc,
    Sim,
    Aka,
    AkaPrime,
}


fn escape_field_value(value: &str) -> String {
    // escape \ first so we don't escape the escape sequences
    let value = value.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('"', "\\\"")
        .replace(':', "\\:");

    if could_be_ascii_hex(&value) {
        format!("\"{value}\"")
    } else {
        value
    }
}

fn could_be_ascii_hex(value: &str) -> bool {
    for c in value.chars() {
        if !"0123456789abcdef".contains(c) {
            return false;
        }
    }
    true
}
