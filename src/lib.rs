use std::fmt::Debug;
use std::fmt::Display;
use std::path::Path;

use arqoii::types::QoiHeader;
use base64::Engine;

#[cfg(feature = "cli")]
use clap::{builder::PossibleValue, ValueEnum};

use image::ImageBuffer;
use image::Luma;
use qrcode::QrCode;
use qrcode::render::Pixel;

#[derive(Clone)]
#[non_exhaustive]
pub enum ImageFormat {
    #[non_exhaustive]
    ImageFormat(image::ImageFormat),
    #[cfg(feature = "qoi")]
    #[non_exhaustive]
    Qoi,
}

impl Debug for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageFormat::ImageFormat(format) => write!(f, "{format:?}"),
            ImageFormat::Qoi => write!(f, "Qoi"),
        }
    }
}

impl Default for ImageFormat {
    fn default() -> Self {
        Self::ImageFormat(image::ImageFormat::Png)
    }
}

#[cfg(feature = "cli")]
impl ValueEnum for ImageFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Qoi, Self::ImageFormat(image::ImageFormat::Png),
            Self::ImageFormat(image::ImageFormat::Jpeg)
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        let name = format!("{self:?}").to_lowercase();
        Some(
            PossibleValue::new(name),
        )
    }
}

impl ImageFormat {
    pub fn png() -> Self {
        Self::ImageFormat(image::ImageFormat::Png)
    }

    #[cfg(feature = "qoi")]
    pub fn qoi() -> Self {
        Self::Qoi
    }
}

struct Image {
    buffer: ImageBuffer<Luma<u8>, Vec<u8>>,
}

impl Image {
    pub fn save(&self, format: ImageFormat, file_path: &Path) -> Result<(), GenerationError> {
        match format {
            ImageFormat::ImageFormat(format) => {
                self.buffer.save_with_format(file_path, format)?;
            }
            ImageFormat::Qoi => {
                let data = arqoii::QoiEncoder::new(
                    QoiHeader::new(
                        self.buffer.width(),
                        self.buffer.height(),
                        arqoii::types::QoiChannels::Rgb,
                        arqoii::types::QoiColorSpace::SRgbWithLinearAlpha,
                    ),
                    self.buffer.pixels().map(|px| arqoii::Pixel {
                        r: px.0[0],
                        g: px.0[0],
                        b: px.0[0],
                        a: 255,
                    }),
                )
                .collect::<Vec<_>>();
                std::fs::write(file_path, data)?;
            }
        }
        Ok(())
    }
    pub fn save_guess_format(&self, file_path: &Path) -> Result<(), GenerationError> {
        if cfg!(feature = "qoi") && file_path.extension().is_some_and(|ext| ext == "qoi") {
            self.save(ImageFormat::Qoi, file_path)
        } else {
            self.buffer.save(file_path)?;
            Ok(())
        }
    }
}


#[derive(Debug, Clone, Copy)]
struct Px(Luma<u8>);

struct Canvas(Px, Image);

impl Pixel for Px {
    type Image = Image;

    type Canvas = Canvas;

    fn default_color(color: qrcode::Color) -> Self {
        Self(Luma([color.select(0, 255)]))
    }
}

impl qrcode::render::Canvas for Canvas {
    type Pixel = Px;

    type Image = <Px as Pixel>::Image;

    fn new(width: u32, height: u32, dark_pixel: Self::Pixel, light_pixel: Self::Pixel) -> Self {
        Self(dark_pixel, Image { buffer: ImageBuffer::from_pixel(width, height, light_pixel.0) })
    }

    fn draw_dark_pixel(&mut self, x: u32, y: u32) {
        self.1.buffer.put_pixel(x, y, self.0.0)
    }

    fn into_image(self) -> Self::Image {
        self.1
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GenerationError {
    #[error("{0}")]
    QrError(#[from] qrcode::types::QrError),
    #[error("{0}")]
    ImageError(#[from] image::error::ImageError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct Wifi {
    ssid: String,
    kind: Option<WifiMethod>,
    hidden: bool,
    eap_method: Option<EapMethod>,
    phase2: Option<Phase2>,
    anonymous_identity: Option<String>,
    identity: Option<String>,
    password: Option<String>,
    public_key: Option<Vec<u8>>,
}

impl Wifi {
    pub fn new(ssid: String) -> Self {
        Self {
            ssid,
            kind: None,
            hidden: false,
            eap_method: None,
            phase2: None,
            anonymous_identity: None,
            identity: None,
            password: None,
            public_key: None,
        }
    }

    pub fn with_method(mut self, wifi_method: Option<WifiMethod>) -> Self {
        self.kind = wifi_method;
        self
    }

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn with_eap_method(mut self, eap: Option<EapMethod>) -> Self {
        self.eap_method = eap;
        self
    }

    pub fn with_phase2(mut self, ph2: Option<Phase2>) -> Self {
        self.phase2 = ph2;
        self
    }

    pub fn with_anonymous_identity(mut self, anon: Option<String>) -> Self {
        self.anonymous_identity = anon;
        self
    }

    pub fn with_identity(mut self, id: Option<String>) -> Self {
        self.identity = id;
        self
    }

    pub fn with_password(mut self, pw: Option<String>) -> Self {
        self.password = pw;
        self
    }

    pub fn with_public_key(mut self, pk: Option<Vec<u8>>) -> Self {
        self.public_key = pk;
        self
    }

    pub fn generate_image_file(
        &self,
        format: Option<ImageFormat>,
        file_path: &Path,
    ) -> Result<(), GenerationError> {
        let code = QrCode::new(self.to_string())?;

        let image = code.render::<Px>().build();

        match format {
            Some(format) => image.save(format, file_path)?,
            None => image.save_guess_format(file_path)?,
        }

        Ok(())
    }

    fn expected_field_count(&self) -> usize {
        self.kind.as_ref().map_or(0, |method|if let WifiMethod::Wpa3 = method {
            2
        } else {
            1
        })
         + 1 // ssid is required
            + self.hidden as usize
            + self.eap_method.is_some() as usize
            + self.phase2.is_some() as usize
            + self.anonymous_identity.is_some() as usize
            + self.identity.is_some() as usize
            + self.password.is_some() as usize
            + self.public_key.is_some() as usize
    }

    fn fields(&self) -> Vec<Field> {
        let expected_fields = self.expected_field_count();

        let mut fields = Vec::with_capacity(expected_fields);

        if let Some(kind) = &self.kind {
            kind.add_fields(&mut fields);
        }

        fields.push(Field::new_string("S", &self.ssid));

        if self.hidden {
            fields.push(Field::new_string("H", "true"))
        }

        if let Some(eap) = &self.eap_method {
            eap.add_fields(&mut fields);
        }

        if let Some(ph2) = &self.phase2 {
            ph2.add_fields(&mut fields)
        }

        if let Some(anon) = &self.anonymous_identity {
            fields.push(Field::new_string("A", anon));
        }

        if let Some(ident) = &self.identity {
            fields.push(Field::new_string("I", ident));
        }

        if let Some(password) = &self.password {
            fields.push(Field::new_string("P", password));
        }

        if let Some(pk) = &self.public_key {
            fields.push(Field::new_base64("K", pk));
        }

        fields
    }
}

impl ToString for Wifi {
    fn to_string(&self) -> String {
        let content: String = self.fields().into_iter().map(|f| f.to_string()).collect();
        format!("WIFI:{content};")
    }
}

pub struct Field {
    name: String,
    value: String,
}

impl Field {
    fn new_string(name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value: Self::escape_field_value(value.as_ref()),
        }
    }

    fn new_base64(name: impl AsRef<str>, value: impl AsRef<[u8]>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value: base64::engine::general_purpose::STANDARD.encode(value),
        }
    }

    fn new_hex(name: impl AsRef<str>, value: impl AsRef<[u8]>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value: value.as_ref().iter().map(|b| format!("{b:x}")).collect(),
        }
    }

    fn escape_field_value(value: &str) -> String {
        // escape \ first so we don't escape the escape sequences
        let value = value
            .replace('\\', "\\\\")
            .replace(';', "\\;")
            .replace(',', "\\,")
            .replace('"', "\\\"")
            .replace(':', "\\:");

        if Self::could_be_ascii_hex(&value) {
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
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{};", self.name, self.value)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[non_exhaustive]
pub enum WifiMethod {
    NoPass,
    Wep,
    /// WPA is also used for WPA2 and WPA3
    Wpa,
    Wpa2Enterprise,
    /// Same as WPA, but for devices that support it includes a flag for WPA2/WPA3 transition mode disabled, to prevent downgrade attacks
    Wpa3,
}

impl WifiMethod {
    pub fn add_fields(&self, fields: &mut Vec<Field>) {
        let kind = match self {
            WifiMethod::NoPass => "nopass",
            WifiMethod::Wep => "WEP",
            WifiMethod::Wpa
            // https://superuser.com/a/1752085
            | WifiMethod::Wpa3 => "WPA",
            WifiMethod::Wpa2Enterprise => "WPA2-EAP",
        };

        fields.push(Field::new_string("T", kind));

        if let WifiMethod::Wpa3 = self {
            // https://superuser.com/a/1752085
            // https://www.wi-fi.org/file/wpa3tm-specification
            // https://www.wi-fi.org/system/files/WPA3%20Specification%20v3.1.pdf
            fields.push(Field::new_hex("R", [1]))
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[non_exhaustive]
pub enum EapMethod {
    Peap,
    Tls,
    Ttls,
    Pwd,
    Sim,
    Aka,
    AkaPrime,
}

impl EapMethod {
    pub fn add_fields(&self, fields: &mut Vec<Field>) {
        let eap_name = match self {
            EapMethod::Peap => "PEAP",
            EapMethod::Tls => "TLS",
            EapMethod::Ttls => "TTLS",
            EapMethod::Pwd => "PWD",
            EapMethod::Sim => "SIM",
            EapMethod::Aka => "AKA",
            EapMethod::AkaPrime => "AKA_PRIME",
        };
        fields.push(Field::new_string("E", eap_name));
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[non_exhaustive]
pub enum Phase2 {
    MsChap,
    MsChapV2,
    Pap,
    Gtc,
    Sim,
    Aka,
    AkaPrime,
}

impl Phase2 {
    pub fn add_fields(&self, fields: &mut Vec<Field>) {
        let ph2_name = match self {
            Phase2::MsChap => "MSCHAP",
            Phase2::MsChapV2 => "MSCHAPV2",
            Phase2::Gtc => "GTC",
            Phase2::Sim => "SIM",
            Phase2::Aka => "AKA",
            Phase2::AkaPrime => "AKA_PRIME",
            Phase2::Pap => "PAP",
        };
        fields.push(Field::new_string("PH2", ph2_name));
    }
}
