use plotly::color;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    NamedColor(color::NamedColor),
    RgbColor(color::Rgb),
    RgbaColor(color::Rgba),
}

impl color::Color for Color {}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        if let Some(s) = value.as_str()
            && let Ok(named) = serde_json::from_str::<color::NamedColor>(&format!("\"{s}\""))
        {
            return Ok(Self::NamedColor(named));
        }

        if let Some(s) = value.as_str()
            && let Some(rgb) = parse_hex_color(s)
        {
            return Ok(Self::RgbColor(rgb));
        }

        if let Ok(rgb) = serde_json::from_value::<color::Rgb>(value.clone()) {
            return Ok(Self::RgbColor(rgb));
        }

        if let Ok(rgba) = serde_json::from_value::<color::Rgba>(value) {
            return Ok(Self::RgbaColor(rgba));
        }

        Err(serde::de::Error::custom("invalid color format"))
    }
}

fn parse_hex_color(value: &str) -> Option<color::Rgb> {
    let hex = value.strip_prefix('#')?;

    let (r, g, b) = match hex.len() {
        3 => {
            let mut chars = hex.chars();
            let r = chars.next()?;
            let g = chars.next()?;
            let b = chars.next()?;
            let rr = u8::from_str_radix(&format!("{r}{r}"), 16).ok()?;
            let gg = u8::from_str_radix(&format!("{g}{g}"), 16).ok()?;
            let bb = u8::from_str_radix(&format!("{b}{b}"), 16).ok()?;
            (rr, gg, bb)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b)
        }
        _ => return None,
    };

    Some(color::Rgb::new(r, g, b))
}
