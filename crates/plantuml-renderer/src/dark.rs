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

/// Цвета, которые автор задаёт именем (`#LightBlue`): без таблицы светлая заливка осталась бы
/// светлой под уже инвертированным светлым текстом.
const NAMED_COLORS: &[(&str, &str)] = &[
    ("aliceblue", "F0F8FF"),
    ("antiquewhite", "FAEBD7"),
    ("aqua", "00FFFF"),
    ("aquamarine", "7FFFD4"),
    ("azure", "F0FFFF"),
    ("beige", "F5F5DC"),
    ("bisque", "FFE4C4"),
    ("black", "000000"),
    ("blanchedalmond", "FFEBCD"),
    ("blue", "0000FF"),
    ("blueviolet", "8A2BE2"),
    ("brown", "A52A2A"),
    ("burlywood", "DEB887"),
    ("cadetblue", "5F9EA0"),
    ("chartreuse", "7FFF00"),
    ("chocolate", "D2691E"),
    ("coral", "FF7F50"),
    ("cornflowerblue", "6495ED"),
    ("cornsilk", "FFF8DC"),
    ("crimson", "DC143C"),
    ("cyan", "00FFFF"),
    ("darkblue", "00008B"),
    ("darkcyan", "008B8B"),
    ("darkgoldenrod", "B8860B"),
    ("darkgray", "A9A9A9"),
    ("darkgreen", "006400"),
    ("darkgrey", "A9A9A9"),
    ("darkkhaki", "BDB76B"),
    ("darkmagenta", "8B008B"),
    ("darkolivegreen", "556B2F"),
    ("darkorange", "FF8C00"),
    ("darkorchid", "9932CC"),
    ("darkred", "8B0000"),
    ("darksalmon", "E9967A"),
    ("darkseagreen", "8FBC8F"),
    ("darkslateblue", "483D8B"),
    ("darkslategray", "2F4F4F"),
    ("darkslategrey", "2F4F4F"),
    ("darkturquoise", "00CED1"),
    ("darkviolet", "9400D3"),
    ("deeppink", "FF1493"),
    ("deepskyblue", "00BFFF"),
    ("dimgray", "696969"),
    ("dimgrey", "696969"),
    ("dodgerblue", "1E90FF"),
    ("firebrick", "B22222"),
    ("floralwhite", "FFFAF0"),
    ("forestgreen", "228B22"),
    ("fuchsia", "FF00FF"),
    ("gainsboro", "DCDCDC"),
    ("ghostwhite", "F8F8FF"),
    ("gold", "FFD700"),
    ("goldenrod", "DAA520"),
    ("gray", "808080"),
    ("green", "008000"),
    ("greenyellow", "ADFF2F"),
    ("grey", "808080"),
    ("honeydew", "F0FFF0"),
    ("hotpink", "FF69B4"),
    ("indianred", "CD5C5C"),
    ("indigo", "4B0082"),
    ("ivory", "FFFFF0"),
    ("khaki", "F0E68C"),
    ("lavender", "E6E6FA"),
    ("lavenderblush", "FFF0F5"),
    ("lawngreen", "7CFC00"),
    ("lemonchiffon", "FFFACD"),
    ("lightblue", "ADD8E6"),
    ("lightcoral", "F08080"),
    ("lightcyan", "E0FFFF"),
    ("lightgoldenrodyellow", "FAFAD2"),
    ("lightgray", "D3D3D3"),
    ("lightgreen", "90EE90"),
    ("lightgrey", "D3D3D3"),
    ("lightpink", "FFB6C1"),
    ("lightsalmon", "FFA07A"),
    ("lightseagreen", "20B2AA"),
    ("lightskyblue", "87CEFA"),
    ("lightslategray", "778899"),
    ("lightslategrey", "778899"),
    ("lightsteelblue", "B0C4DE"),
    ("lightyellow", "FFFFE0"),
    ("lime", "00FF00"),
    ("limegreen", "32CD32"),
    ("linen", "FAF0E6"),
    ("magenta", "FF00FF"),
    ("maroon", "800000"),
    ("mediumaquamarine", "66CDAA"),
    ("mediumblue", "0000CD"),
    ("mediumorchid", "BA55D3"),
    ("mediumpurple", "9370DB"),
    ("mediumseagreen", "3CB371"),
    ("mediumslateblue", "7B68EE"),
    ("mediumspringgreen", "00FA9A"),
    ("mediumturquoise", "48D1CC"),
    ("mediumvioletred", "C71585"),
    ("midnightblue", "191970"),
    ("mintcream", "F5FFFA"),
    ("mistyrose", "FFE4E1"),
    ("moccasin", "FFE4B5"),
    ("navajowhite", "FFDEAD"),
    ("navy", "000080"),
    ("oldlace", "FDF5E6"),
    ("olive", "808000"),
    ("olivedrab", "6B8E23"),
    ("orange", "FFA500"),
    ("orangered", "FF4500"),
    ("orchid", "DA70D6"),
    ("palegoldenrod", "EEE8AA"),
    ("palegreen", "98FB98"),
    ("paleturquoise", "AFEEEE"),
    ("palevioletred", "DB7093"),
    ("papayawhip", "FFEFD5"),
    ("peachpuff", "FFDAB9"),
    ("peru", "CD853F"),
    ("pink", "FFC0CB"),
    ("plum", "DDA0DD"),
    ("powderblue", "B0E0E6"),
    ("purple", "800080"),
    ("rebeccapurple", "663399"),
    ("red", "FF0000"),
    ("rosybrown", "BC8F8F"),
    ("royalblue", "4169E1"),
    ("saddlebrown", "8B4513"),
    ("salmon", "FA8072"),
    ("sandybrown", "F4A460"),
    ("seagreen", "2E8B57"),
    ("seashell", "FFF5EE"),
    ("sienna", "A0522D"),
    ("silver", "C0C0C0"),
    ("skyblue", "87CEEB"),
    ("slateblue", "6A5ACD"),
    ("slategray", "708090"),
    ("slategrey", "708090"),
    ("snow", "FFFAFA"),
    ("springgreen", "00FF7F"),
    ("steelblue", "4682B4"),
    ("tan", "D2B48C"),
    ("teal", "008080"),
    ("thistle", "D8BFD8"),
    ("tomato", "FF6347"),
    ("turquoise", "40E0D0"),
    ("violet", "EE82EE"),
    ("wheat", "F5DEB3"),
    ("white", "FFFFFF"),
    ("whitesmoke", "F5F5F5"),
    ("yellow", "FFFF00"),
    ("yellowgreen", "9ACD32"),
];

fn named(color: &str) -> Option<String> {
    let lower = color.to_ascii_lowercase();
    let (_, hex) = NAMED_COLORS.iter().find(|(name, _)| *name == lower)?;
    invert(hex)
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
    for attribute in ["fill", "stroke"] {
        let prefix = format!("{attribute}=\"");
        let Some(start) = out.find(&prefix).map(|at| at + prefix.len()) else {
            continue;
        };
        let Some(end) = out[start..].find('"').map(|len| start + len) else {
            continue;
        };
        if let Some(value) = named(&out[start..end]) {
            out.replace_range(start..end, &value);
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
    fn test_named_colors_invert_in_any_case() {
        let out = apply(r#"<rect fill="LightBlue" stroke="black"/><rect fill="none"/>"#);
        assert!(!out.contains("LightBlue"), "{out}");
        assert!(out.contains(r##"stroke="#FFFFFF""##), "{out}");
        assert!(out.contains(r#"fill="none""#), "{out}");
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
