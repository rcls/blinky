pub const SYMBOLS: &str =
    " 𜺨𜺫🮂𜴀▘𜴁𜴂𜴃𜴄▝𜴅𜴆𜴇𜴈▀\
     𜴉𜴊𜴋𜴌🯦𜴍𜴎𜴏𜴐𜴑𜴒𜴓𜴔𜴕𜴖𜴗\
     𜴘𜴙𜴚𜴛𜴜𜴝𜴞𜴟🯧𜴠𜴡𜴢𜴣𜴤𜴥𜴦\
     𜴧𜴨𜴩𜴪𜴫𜴬𜴭𜴮𜴯𜴰𜴱𜴲𜴳𜴴𜴵🮅\
     𜺣𜴶𜴷𜴸𜴹𜴺𜴻𜴼𜴽𜴾𜴿𜵀𜵁𜵂𜵃𜵄\
     ▖𜵅𜵆𜵇𜵈▌𜵉𜵊𜵋𜵌▞𜵍𜵎𜵏𜵐▛\
     𜵑𜵒𜵓𜵔𜵕𜵖𜵗𜵘𜵙𜵚𜵛𜵜𜵝𜵞𜵟𜵠\
     𜵡𜵢𜵣𜵤𜵥𜵦𜵧𜵨𜵩𜵪𜵫𜵬𜵭𜵮𜵯𜵰\
     𜺠𜵱𜵲𜵳𜵴𜵵𜵶𜵷𜵸𜵹𜵺𜵻𜵼𜵽𜵾𜵿\
     𜶀𜶁𜶂𜶃𜶄𜶅𜶆𜶇𜶈𜶉𜶊𜶋𜶌𜶍𜶎𜶏\
     ▗𜶐𜶑𜶒𜶓▚𜶔𜶕𜶖𜶗▐𜶘𜶙𜶚𜶛▜\
     𜶜𜶝𜶞𜶟𜶠𜶡𜶢𜶣𜶤𜶥𜶦𜶧𜶨𜶩𜶪𜶫\
     ▂𜶬𜶭𜶮𜶯𜶰𜶱𜶲𜶳𜶴𜶵𜶶𜶷𜶸𜶹𜶺\
     𜶻𜶼𜶽𜶾𜶿𜷀𜷁𜷂𜷃𜷄𜷅𜷆𜷇𜷈𜷉𜷊\
     𜷋𜷌𜷍𜷎𜷏𜷐𜷑𜷒𜷓𜷔𜷕𜷖𜷗𜷘𜷙𜷚\
     ▄𜷛𜷜𜷝𜷞▙𜷟𜷠𜷡𜷢▟𜷣▆𜷤𜷥█";

const POS: [u16; 257] = {
    let mut me = [0; _];
    let mut i = 0;
    let mut pos = 0;
    while i < 257 {
        me[i] = pos as u16;
        i += 1;
        pos = SYMBOLS.ceil_char_boundary(pos + 1);
    }
    me
};

fn blocky(b: u8) -> &'static str {
    &SYMBOLS[POS[b as usize] as usize .. POS[b as usize + 1] as usize]
}

pub fn blocks(display: u64) -> [&'static str; 6] {
    fn spread4(x: u64) -> u8 {
        let x = x as u8;
        x & 1 | x << 1 & 4 | x << 2 & 16 | x << 3 & 64
    }
    let interleave = |n| spread4(display >> n) | spread4(display >> n + 8) << 1;

    let indexes = [0, 16, 32, 4, 20, 36];
    core::array::from_fn(|i| blocky(interleave(indexes[i])))
}

#[cfg(test)]
fn test_text<const N: usize>(text: &[u8; N]) {
    let text = crate::chars::map_str(text);
    for row in 0 ..= 1 {
        for c in text {
            let b = blocks(crate::chars::COLUMNS[c as usize]);
            for i in row * 3 .. row * 3 + 3 {
                print!("{}", b[i]);
            }
        }
        println!();
    }
}

#[test]
fn text1() {
    test_text(b"THIS IS A TEST");
    test_text(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    test_text(b"0123456789");
}
