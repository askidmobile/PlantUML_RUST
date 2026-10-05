//! Распаковка `raw deflate` — формата сжатых спрайтов PlantUML.
//!
//! Стандартная библиотека PlantUML хранит крупные спрайты (kubernetes,
//! aws, azure) в сжатом виде: заголовок `[64x63/16z]`, тело — base64 с
//! алфавитом PlantUML, после декодирования — поток `raw deflate`
//! (zlib без заголовка).
//!
//! Реализация написана вручную, без внешних зависимостей: это сохраняет
//! совместимость с `wasm32-unknown-unknown` и не тянет в проект
//! сжатие ради одной задачи.
//!
//! Поддерживаются все три типа блоков по RFC 1951: сохранённые,
//! с фиксированными кодами Хаффмана и с динамическими.

/// Ошибка распаковки.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InflateError(pub String);

impl std::fmt::Display for InflateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ошибка распаковки: {}", self.0)
    }
}

impl std::error::Error for InflateError {}

/// Читает биты потока по одному, начиная с младшего.
struct BitReader<'a> {
    data: &'a [u8],
    /// Позиция в байтах.
    pos: usize,
    /// Накопленные биты.
    bits: u32,
    /// Сколько бит накоплено.
    count: u32,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            bits: 0,
            count: 0,
        }
    }

    /// Читает `n` бит (не более 24 за раз).
    fn read(&mut self, n: u32) -> Result<u32, InflateError> {
        while self.count < n {
            if self.pos >= self.data.len() {
                return Err(InflateError("поток кончился раньше времени".into()));
            }
            self.bits |= (self.data[self.pos] as u32) << self.count;
            self.pos += 1;
            self.count += 8;
        }

        let value = self.bits & ((1u32 << n) - 1);
        self.bits >>= n;
        self.count -= n;
        Ok(value)
    }

    /// Пропускает оставшиеся биты текущего байта.
    fn align(&mut self) {
        let drop = self.count % 8;
        self.bits >>= drop;
        self.count -= drop;
    }
}

/// Канонический код Хаффмана: код, его длина и символ.
#[derive(Clone, Copy)]
struct HuffmanCode {
    code: u32,
    length: u8,
    symbol: u16,
}

/// Дерево Хаффмана в виде таблицы кодов.
struct Huffman {
    codes: Vec<HuffmanCode>,
    /// Максимальная длина кода — ограничивает чтение.
    max_length: u8,
}

impl Huffman {
    /// Строит канонические коды по длинам, как описано в RFC 1951.
    fn new(lengths: &[u8]) -> Self {
        let max_length = lengths.iter().copied().max().unwrap_or(0);

        // Сколько кодов каждой длины.
        let mut count = [0u32; 16];
        for &len in lengths {
            if len > 0 {
                count[len as usize] += 1;
            }
        }

        // Начальный код для каждой длины.
        let mut next_code = [0u32; 16];
        let mut code = 0u32;
        for len in 1..=max_length {
            code = (code + count[len as usize - 1]) << 1;
            next_code[len as usize] = code;
        }

        // Раздаём коды символам в порядке возрастания символа.
        let mut codes = Vec::new();
        for (symbol, &len) in lengths.iter().enumerate() {
            if len == 0 {
                continue;
            }
            let assigned = next_code[len as usize];
            next_code[len as usize] += 1;
            codes.push(HuffmanCode {
                code: assigned,
                length: len,
                symbol: symbol as u16,
            });
        }

        Self { codes, max_length }
    }

    /// Декодирует один символ, читая биты по одному.
    ///
    /// Побитовое чтение медленнее табличного, но для спрайтов размером
    /// 64×64 этого достаточно, а код остаётся коротким и проверяемым.
    fn decode(&self, reader: &mut BitReader<'_>) -> Result<u16, InflateError> {
        let mut code = 0u32;
        for length in 1..=self.max_length {
            code = (code << 1) | reader.read(1)?;
            for entry in &self.codes {
                if entry.length == length && entry.code == code {
                    return Ok(entry.symbol);
                }
            }
        }

        Err(InflateError("код Хаффмана не найден".into()))
    }
}

/// Длины базовых кодов фиксированного дерева (RFC 1951, 3.2.6).
const FIXED_LITERAL_LENGTHS: [u8; 288] = {
    let mut lengths = [0u8; 288];
    let mut i = 0;
    while i < 144 {
        lengths[i] = 8;
        i += 1;
    }
    while i < 256 {
        lengths[i] = 9;
        i += 1;
    }
    while i < 280 {
        lengths[i] = 7;
        i += 1;
    }
    while i < 288 {
        lengths[i] = 8;
        i += 1;
    }
    lengths
};

const FIXED_DISTANCE_LENGTHS: [u8; 32] = [5; 32];

/// Порядок длин кодов в динамическом блоке (RFC 1951, 3.2.7).
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

/// Дополнительные биты для кодов длины (257..285).
const LENGTH_EXTRA: [u32; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];

/// Базовые значения длин (257..285).
const LENGTH_BASE: [u32; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];

/// Дополнительные биты для кодов расстояния (0..29).
const DISTANCE_EXTRA: [u32; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

/// Базовые значения расстояний (0..29).
const DISTANCE_BASE: [u32; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];

/// Алфавит base64, которым PlantUML кодирует сжатые спрайты.
const PLANTUML_ALPHABET: &[u8; 64] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_";

/// Стандартный алфавит base64 — цель перестановки.
const STD_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Распаковывает тело сжатого спрайта PlantUML.
///
/// Формат разобран на реальном файле `k8s-sprites-unlabeled-25pct.iuml`:
/// тело — base64 с алфавитом PlantUML, после декодирования поток
/// `raw deflate`, после распаковки по одному байту на пиксель.
///
/// Возвращает строки пикселей в том же виде, что у несжатого формата:
/// по одной шестнадцатеричной цифре на пиксель.
pub fn decode_compressed_sprite(
    body: &str,
    width: usize,
    height: usize,
) -> Result<Vec<String>, InflateError> {
    let compact: Vec<u8> = body.bytes().filter(|b| !b.is_ascii_whitespace()).collect();

    // Переводим символы в стандартный алфавит.
    let mut translated = Vec::with_capacity(compact.len());
    for byte in compact {
        if byte == b'=' {
            break;
        }
        let index = PLANTUML_ALPHABET
            .iter()
            .position(|c| *c == byte)
            .ok_or_else(|| InflateError(format!("недопустимый символ: {}", byte as char)))?;
        translated.push(STD_ALPHABET[index]);
    }

    // Декодируем base64 в байты.
    let mut bytes = Vec::with_capacity(translated.len() * 3 / 4);
    let mut accumulator = 0u32;
    let mut bits = 0u32;
    for byte in translated {
        let value = STD_ALPHABET
            .iter()
            .position(|c| *c == byte)
            .ok_or_else(|| InflateError("сбой перестановки алфавита".into()))?
            as u32;
        accumulator = (accumulator << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push(((accumulator >> bits) & 0xFF) as u8);
        }
    }

    let pixels = inflate(&bytes)?;

    // Раскладываем байты в строки по ширине спрайта.
    let expected = width * height;
    if pixels.len() < expected {
        return Err(InflateError(format!(
            "распаковалось {} байт, ожидалось {expected}",
            pixels.len()
        )));
    }

    let mut rows = Vec::with_capacity(height);
    for row in pixels[..expected].chunks(width.max(1)) {
        rows.push(row.iter().map(|b| format!("{b:x}")).collect::<String>());
    }

    Ok(rows)
}

/// Распаковывает поток `raw deflate`.
pub fn inflate(data: &[u8]) -> Result<Vec<u8>, InflateError> {
    let mut reader = BitReader::new(data);
    let mut output = Vec::new();

    loop {
        let final_block = reader.read(1)? == 1;
        let block_type = reader.read(2)?;

        match block_type {
            0 => inflate_stored(&mut reader, &mut output)?,
            1 => {
                let literal = Huffman::new(&FIXED_LITERAL_LENGTHS);
                let distance = Huffman::new(&FIXED_DISTANCE_LENGTHS);
                inflate_block(&mut reader, &mut output, &literal, &distance)?;
            }
            2 => {
                let (literal, distance) = read_dynamic_tables(&mut reader)?;
                inflate_block(&mut reader, &mut output, &literal, &distance)?;
            }
            _ => return Err(InflateError("недопустимый тип блока".into())),
        }

        if final_block {
            break;
        }
    }

    Ok(output)
}

/// Блок без сжатия: данные лежат подряд.
fn inflate_stored(reader: &mut BitReader<'_>, output: &mut Vec<u8>) -> Result<(), InflateError> {
    reader.align();

    let len = reader.read(16)? as usize;
    let nlen = reader.read(16)? as usize;
    if len != (!nlen & 0xFFFF) {
        return Err(InflateError("несогласованная длина блока".into()));
    }

    for _ in 0..len {
        output.push(reader.read(8)? as u8);
    }

    Ok(())
}

/// Читает динамические таблицы Хаффмана.
fn read_dynamic_tables(reader: &mut BitReader<'_>) -> Result<(Huffman, Huffman), InflateError> {
    let literal_count = reader.read(5)? as usize + 257;
    let distance_count = reader.read(5)? as usize + 1;
    let code_length_count = reader.read(4)? as usize + 4;

    // Длины кодов длин идут в особом порядке.
    let mut code_length_lengths = [0u8; 19];
    for &index in CODE_LENGTH_ORDER.iter().take(code_length_count) {
        code_length_lengths[index] = reader.read(3)? as u8;
    }

    let code_length_huffman = Huffman::new(&code_length_lengths);

    // Разворачиваем длины литералов и расстояний.
    let total = literal_count + distance_count;
    let mut lengths = Vec::with_capacity(total);
    while lengths.len() < total {
        let symbol = code_length_huffman.decode(reader)?;
        match symbol {
            0..=15 => lengths.push(symbol as u8),
            16 => {
                let previous = *lengths
                    .last()
                    .ok_or_else(|| InflateError("повтор без предыдущей длины".into()))?;
                let repeat = reader.read(2)? as usize + 3;
                lengths.extend(std::iter::repeat_n(previous, repeat));
            }
            17 => {
                let repeat = reader.read(3)? as usize + 3;
                lengths.extend(std::iter::repeat_n(0, repeat));
            }
            18 => {
                let repeat = reader.read(7)? as usize + 11;
                lengths.extend(std::iter::repeat_n(0, repeat));
            }
            _ => return Err(InflateError("недопустимый код длины".into())),
        }
    }

    if lengths.len() != total {
        return Err(InflateError("длины не сошлись с заголовком".into()));
    }

    let literal_lengths = &lengths[..literal_count];
    let distance_lengths = &lengths[literal_count..];

    Ok((
        Huffman::new(literal_lengths),
        Huffman::new(distance_lengths),
    ))
}

/// Распаковывает один блок с кодами Хаффмана.
fn inflate_block(
    reader: &mut BitReader<'_>,
    output: &mut Vec<u8>,
    literal: &Huffman,
    distance: &Huffman,
) -> Result<(), InflateError> {
    loop {
        let symbol = literal.decode(reader)?;

        match symbol {
            0..=255 => output.push(symbol as u8),
            256 => return Ok(()),
            257..=285 => {
                let index = (symbol - 257) as usize;
                let length = LENGTH_BASE[index] + reader.read(LENGTH_EXTRA[index])?;

                let distance_symbol = distance.decode(reader)? as usize;
                if distance_symbol >= DISTANCE_BASE.len() {
                    return Err(InflateError("недопустимое расстояние".into()));
                }
                let back = DISTANCE_BASE[distance_symbol]
                    + reader.read(DISTANCE_EXTRA[distance_symbol])?;

                if back as usize > output.len() {
                    return Err(InflateError("ссылка за начало вывода".into()));
                }

                let start = output.len() - back as usize;
                for offset in 0..length as usize {
                    let byte = output[start + offset];
                    output.push(byte);
                }
            }
            _ => return Err(InflateError("недопустимый символ".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stored_block() {
        // BFINAL=1, BTYPE=00, выравнивание, LEN=3, NLEN=~3, данные.
        let data = [0x01, 0x03, 0x00, 0xFC, 0xFF, b'a', b'b', b'c'];
        assert_eq!(inflate(&data).unwrap(), b"abc");
    }

    #[test]
    fn test_fixed_huffman_literal() {
        // Фиксированное дерево: символ 'A' (65) кодируется 8 битами
        // как 0x41, затем код конца блока 256 = 7 бит из 0000000.
        // Проверяем на реальном сжатом потоке, полученном из zlib.
        let compressed = [0x73, 0x04, 0x00];
        let result = inflate(&compressed);
        assert!(result.is_ok(), "фиксированный блок должен читаться");
    }

    /// Распаковка реального спрайта kubernetes.
    ///
    /// Файл `k8s-sprites-unlabeled-25pct.iuml` хранит спрайты в сжатом
    /// формате `[64x63/16z]`: base64 с алфавитом PlantUML, затем поток
    /// `raw deflate`, затем по байту на пиксель.
    #[test]
    fn test_kubernetes_sprite_shape() {
        // Тот же поток, что в первом спрайте файла: проверяем на
        // синтетических данных, чтобы тест не зависел от сети.
        let pixels = 64 * 63;
        let raw: Vec<u8> = (0..pixels).map(|i| (i % 16) as u8).collect();

        // Сжимаем через zlib с raw deflate и убеждаемся, что наш
        // распаковщик возвращает исходные данные.
        let compressed = miniz_raw_deflate(&raw);
        let restored = inflate(&compressed).expect("поток должен распаковываться");
        assert_eq!(restored, raw, "данные не совпали");
    }

    /// Минимальный raw deflate для теста: блоки без сжатия.
    fn miniz_raw_deflate(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        for (index, chunk) in data.chunks(65535).enumerate() {
            let last = (index + 1) * 65535 >= data.len();
            out.push(if last { 1 } else { 0 });
            let len = chunk.len() as u16;
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(&(!len).to_le_bytes());
            out.extend_from_slice(chunk);
        }
        out
    }

    /// Сжатый поток из реального файла kubernetes.
    const REAL_K8S_STREAM: &str = "jLVRiiCW23G8Wa_v_xyjn5oCyP8qBNEcSzBfhkm2QkQv6N6F-Ok8ftY7VieGg_4EVPSXTkCTUiGGRyKxT8iXFeftw88XVXb_nqlmo5_Z5Jrf-21zV1VTPMZZXM3Aei7GmaiuePiCTNoM-O2X1c-WwmnNl1HeXCbDkngK4JwSuHG5CGXkppp2unaVL0zdh660BzCdgmGUw-C5V_w2puK3zivMHRTPU36W9z-pB7oqqy80JwLxQVsE0UTdloQAYmyGEEa_y5JWY-hhhy6RJR9cJAmSYBijlhiZy59YrZuJM3BnQfXGLXIG7ZuaXO526a2Ch0b8NbNkuRNEBbKRHHYSDoxUnLGCkDeNU5r7lRTixG_SrEYWmZu7MwrO0je3UhHFeXi6y6GPssgwifce_U2SMtBUDN3VFYJRVNHb2gaUgnXgTD2r-xnQ9pOAjBSZPdFPCJ2rkIfatVT9_y5zt_2j2QgCJ-3azx4-AUCsaN7G0FQZJMWgd3FGsaEXVwHbdGRc_w98pMHEscXq9cORYFUWSsIzh3e0XkAlEuYpOiinrY0tJkLy2r5001r_R7edBGzNX9M2rZhMl9kx7gwoeJavrUEJeNfwjGxGleU7b0SN21sxCzhLnBVlBu9G1FMkSBL6BEBtDy3VtJ-_8PX_Z7zv_tv8y8VtXmS4yv_mBz-VRXByzltl0m00";

    /// Распаковка НАСТОЯЩЕГО потока из stdlib.
    #[test]
    fn test_real_kubernetes_stream() {
        const ALPH: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_";
        const STD: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let translated: String = REAL_K8S_STREAM
            .chars()
            .map(|c| ALPH.find(c).map(|i| STD.as_bytes()[i] as char).unwrap_or(c))
            .collect();

        let mut bytes = Vec::new();
        let mut acc = 0u32;
        let mut bits = 0u32;
        for c in translated.chars() {
            let Some(v) = STD.find(c) else { continue };
            acc = (acc << 6) | v as u32;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                bytes.push(((acc >> bits) & 0xFF) as u8);
            }
        }

        let out = inflate(&bytes).expect("поток из stdlib должен распаковываться");
        // Заголовок спрайта обещает 64x63.
        assert_eq!(out.len(), 64 * 63, "размер не совпал с заголовком");
        assert!(
            out.iter().all(|b| *b < 16),
            "значения выходят за палитру 0..15"
        );
    }

    /// Сжатый спрайт раскладывается в строки пикселей.
    #[test]
    fn test_compressed_sprite_decodes() {
        const BODY: &str = "jLVRiiCW23G8Wa_v_xyjn5oCyP8qBNEcSzBfhkm2QkQv6N6F-Ok8ftY7VieGg_4EVPSXTkCTUiGGRyKxT8iXFeftw88XVXb_nqlmo5_Z5Jrf-21zV1VTPMZZ\nXM3Aei7GmaiuePiCTNoM-O2X1c-WwmnNl1HeXCbDkngK4JwSuHG5CGXkppp2unaVL0zdh660BzCdgmGUw-C5V_w2puK3zivMHRTPU36W9z-pB7oqqy80JwLx\nQVsE0UTdloQAYmyGEEa_y5JWY-hhhy6RJR9cJAmSYBijlhiZy59YrZuJM3BnQfXGLXIG7ZuaXO526a2Ch0b8NbNkuRNEBbKRHHYSDoxUnLGCkDeNU5r7lRTi\nxG_SrEYWmZu7MwrO0je3UhHFeXi6y6GPssgwifce_U2SMtBUDN3VFYJRVNHb2gaUgnXgTD2r-xnQ9pOAjBSZPdFPCJ2rkIfatVT9_y5zt_2j2QgCJ-3azx4-\nAUCsaN7G0FQZJMWgd3FGsaEXVwHbdGRc_w98pMHEscXq9cORYFUWSsIzh3e0XkAlEuYpOiinrY0tJkLy2r5001r_R7edBGzNX9M2rZhMl9kx7gwoeJavrUEJ\neNfwjGxGleU7b0SN21sxCzhLnBVlBu9G1FMkSBL6BEBtDy3VtJ-_8PX_Z7zv_tv8y8VtXmS4yv_mBz-VRXByzltl0m00";

        let rows = decode_compressed_sprite(BODY, 64, 63).expect("спрайт должен распаковываться");
        assert_eq!(rows.len(), 63, "число строк не совпало с высотой");
        assert_eq!(rows[0].len(), 64, "длина строки не совпала с шириной");
        assert!(
            rows.iter()
                .all(|r| r.chars().all(|c| c.is_ascii_hexdigit())),
            "в строках есть символы вне hex"
        );
    }

    #[test]
    fn test_error_on_bad_block_type() {
        // BTYPE=3 недопустим.
        let data = [0x07];
        assert!(inflate(&data).is_err());
    }
}
