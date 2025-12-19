use log::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub(crate) struct AsciiFormChanger {
    zenkaku_map: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct AsciiFormMap {
    #[serde(default)]
    hankaku: Vec<String>,
    #[serde(default)]
    zenkaku: Vec<String>,
}

impl AsciiFormChanger {
    pub fn default_ascii_form_changer() -> Self {
        let base_dirs = xdg::BaseDirectories::new();

        if let Some(filepath) = base_dirs.find_data_file("libcskk/rule/ascii_form.toml") {
            AsciiFormChanger::from_path(&filepath)
        } else {
            AsciiFormChanger::from_string("")
        }
    }

    pub fn from_path(filepath: &Path) -> Self {
        let mut file = File::open(filepath).expect("file not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("file read error");
        AsciiFormChanger::from_string(&contents)
    }

    /// Kept for backward compatibility. Use from_path instead.
    pub fn from_file(filename: &str) -> Self {
        Self::from_path(Path::new(filename))
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

    #[test]
    fn empty_ascii_form_changer() {
        AsciiFormChanger::from_string("");
    }
}
