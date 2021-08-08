use crate::env::filepath_from_xdg_data_dir;
use log::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

pub(crate) struct AsciiFormChanger {
    zenkaku_map: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct AsciiFormMap {
    hankaku: Vec<String>,
    zenkaku: Vec<String>,
}

impl AsciiFormChanger {
    pub fn default_ascii_form_changer() -> Self {
        let filepath = filepath_from_xdg_data_dir("libcskk/rule/ascii_form.toml");

        if let Ok(filepath) = filepath {
            AsciiFormChanger::from_file(&filepath)
        } else {
            AsciiFormChanger::from_string("")
        }
    }

    pub fn from_file(filename: &str) -> Self {
        let mut file = File::open(filename).expect("file not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("file read error");
        AsciiFormChanger::from_string(&contents)
    }

    fn from_string(contents: &str) -> Self {
        let ascii_form_map: AsciiFormMap =
            toml::from_str(contents).expect("source data file for ascii form is broken");

        if ascii_form_map.zenkaku.len() != ascii_form_map.hankaku.len() {
            warn!("source data file for ascii form doesn't match in length");
            return Self {
                zenkaku_map: BTreeMap::new(),
            };
        }
        let mut zenkaku_map = BTreeMap::new();
        let mut zenkaku_iter = ascii_form_map.zenkaku.iter();
        for ascii_char in ascii_form_map.hankaku {
            zenkaku_map.insert(ascii_char, zenkaku_iter.next().unwrap().to_owned());
        }
        Self { zenkaku_map }
    }

    pub(crate) fn adjust_ascii_char(&self, ascii_char: char) -> String {
        if let Some(zenkaku_ascii) = self.zenkaku_map.get(&ascii_char.to_string()) {
            zenkaku_ascii.to_owned()
        } else {
            ascii_char.to_string()
        }
    }

    #[allow(dead_code)]
    pub(crate) fn adjust_ascii_string(&self, ascii_string: &str) -> String {
        let mut result = String::new();
        for c in ascii_string.chars() {
            if let Some(zenkaku_ascii) = self.zenkaku_map.get(&c.to_string()) {
                result.push_str(zenkaku_ascii)
            } else {
                result.push(c);
            };
        }
        result
    }
}

#[cfg(test)]
impl AsciiFormChanger {
    pub fn test_ascii_form_changer() -> Self {
        AsciiFormChanger::from_string(
            "\
hankaku = [\" \", \"!\", \"\\\"\", \"a\", \"b\", \"1\", \"\\\\\"]
zenkaku = [\"　\", \"！\", \"”\", \"ａ\", \"ｂ\", \"１\", \"＼\"]
",
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sanity_check() {
        let changer = AsciiFormChanger::test_ascii_form_changer();
        assert_eq!(changer.adjust_ascii_char('a'), "ａ");
        assert_eq!(changer.adjust_ascii_char('1'), "１");
        assert_eq!(changer.adjust_ascii_char('!'), "！");
    }
}
