//! Тёмная тема: инверсия яркости готового SVG с сохранением оттенка.
//!
//! Цвета задаются в десятках мест рендерера и в самой схеме, поэтому тема применяется
//! к результату: светлые заливки становятся тёмными, тёмные линии и текст светлыми,
//! а контраст и оттенки пользовательских цветов сохраняются.

/// Инвертирует яркость шестнадцатеричного цвета в HSL.
fn invert(hex: &str) -> Option<String> {
    let digits: Vec<u8> = match hex.len() {
        3 => hex
            .chars()
            .map(|c| c.to_digit(16).map(|d| (d * 17) as u8))
            .collect::<Option<Vec<u8>>>()?,
        6 => (0..3)
            .map(|i| u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok())
            .collect::<Option<Vec<u8>>>()?,
        _ => return None,
    };
    let (r, g, b) = (
        f64::from(digits[0]) / 255.0,
        f64::from(digits[1]) / 255.0,
        f64::from(digits[2]) / 255.0,
    );
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let chroma = max - min;
    let lightness = (max + min) / 2.0;
    let target = 1.0 - lightness;
    // Тот же оттенок при новой светлоте: сдвигаем каналы и сохраняем насыщенность.
    let saturation = if chroma == 0.0 {
        0.0
    } else {
        chroma / (1.0 - (2.0 * lightness - 1.0).abs())
    };
    let new_chroma = saturation * (1.0 - (2.0 * target - 1.0).abs());
    let shift = |channel: f64| {
        let unit = if chroma == 0.0 {
            0.5
        } else {
            (channel - min) / chroma
        };
        let value = target + new_chroma * (unit - 0.5);
        (value.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    Some(format!("#{:02X}{:02X}{:02X}", shift(r), shift(g), shift(b)))
}

/// Именованные цвета, встречающиеся в выводе.
fn named(color: &str) -> Option<&'static str> {
    match color {
        "black" => Some("#FFFFFF"),
        "white" => Some("#000000"),
        _ => None,
    }
}

/// Заменяет цвета во всех атрибутах и стилях, кроме содержимого подписей.
pub fn apply(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    // Текст подписей не трогаем: `#123` в нём — это не цвет.
    while let Some(open) = rest.find('<') {
        out.push_str(&rest[..open]);
        let tail = &rest[open..];
        let Some(close) = tail.find('>') else {
            out.push_str(tail);
            return out;
        };
        out.push_str(&convert_tag(&tail[..=close]));
        rest = &tail[close + 1..];
    }
    out.push_str(rest);
    out
}

fn convert_tag(tag: &str) -> String {
    let mut out = String::with_capacity(tag.len());
    let chars: Vec<char> = tag.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '#' {
            let hex: String = chars[i + 1..]
                .iter()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect();
            // Ссылки `url(#id)` и `href="#id"` — не цвета.
            let is_ref = out.ends_with("url(") || out.ends_with("href=\"");
            if let Some(color) = (!is_ref).then(|| invert(&hex)).flatten() {
                out.push_str(&color);
                i += 1 + hex.chars().count();
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    for name in ["black", "white"] {
        if let Some(value) = named(name) {
            for attribute in ["fill", "stroke"] {
                out = out.replace(
                    &format!("{attribute}=\"{name}\""),
                    &format!("{attribute}=\"{value}\""),
                );
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grays_invert_and_hue_stays() {
        assert_eq!(invert("000"), Some("#FFFFFF".to_string()));
        assert_eq!(invert("FFFFFF"), Some("#000000".to_string()));
        assert_eq!(invert("F1F1F1"), Some("#0E0E0E".to_string()));
        // Светло-голубой становится тёмно-голубым: синий канал остаётся старшим.
        let Some(color) = invert("ADD8E6") else {
            panic!("цвет не разобран");
        };
        let blue = u8::from_str_radix(&color[5..7], 16).unwrap_or(0);
        let red = u8::from_str_radix(&color[1..3], 16).unwrap_or(255);
        assert!(blue > red, "{color}");
    }

    #[test]
    fn test_attributes_change_and_text_stays() {
        let out = apply(
            r##"<svg style="background:#FFF;"><text fill="#000">#123 issue</text><use href="#abc"/></svg>"##,
        );
        assert!(out.contains("background:#000000"), "{out}");
        assert!(out.contains(r##"fill="#FFFFFF""##), "{out}");
        assert!(out.contains("#123 issue"), "{out}");
        assert!(out.contains(r##"href="#abc""##), "{out}");
    }
}
