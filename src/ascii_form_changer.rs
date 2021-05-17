use crate::env::filepath_from_xdg_data_dir;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

pub(crate) struct AsciiFormChanger {
    map: AsciiFormMap,
}

#[derive(Deserialize)]
struct AsciiFormMap {
    zenkaku: BTreeMap<String, String>,
}

impl AsciiFormChanger {
    pub fn default_ascii_form_changer() -> Self {
        let filepath = filepath_from_xdg_data_dir("libcskk/rule/ascii_form.toml");

        if let Ok(filepath) = filepath {
            AsciiFormChanger::from_file(&filepath)
        } else {
            AsciiFormChanger::from_string(&"")
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
            toml::from_str(&contents).expect("source data file for kana form is broken");

        Self {
            map: ascii_form_map,
        }
    }

    pub fn adjust_ascii_char(&self, ascii_char: char) -> String {
        if let Some(zenkaku_ascii) = self.map.zenkaku.get(&ascii_char.to_string()) {
            zenkaku_ascii.to_owned()
        } else {
            ascii_char.to_string()
        }
    }
}

#[cfg(test)]
impl AsciiFormChanger {
    pub fn test_ascii_form_changer() -> Self {
        AsciiFormChanger::from_string(
            &"\
[zenkaku]
\"a\" = \"ａ\"
\"b\" = \"ｂ\"
\"1\" = \"１\"
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
    }
}
