// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvSnapshot {
    pub hunspell_dict_dir: Option<String>,
    pub xdg_data_home: Option<String>,
    pub home: Option<String>,
    pub xdg_data_dirs: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemDictionary {
    pub tag: String,
    pub aff_path: PathBuf,
    pub dic_path: PathBuf,
}

pub fn canonicalize_tag(raw: &str) -> String {
    raw.chars()
        .map(|ch| match ch {
            '_' | '-' => '-',
            _ => ch.to_ascii_lowercase(),
        })
        .collect()
}

pub fn build_search_path(env: &EnvSnapshot) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(raw) = &env.hunspell_dict_dir {
        dirs.extend(
            raw.split(':')
                .filter(|part| !part.is_empty())
                .map(PathBuf::from),
        );
    }
    if let Some(xdg_home) = &env.xdg_data_home {
        dirs.push(Path::new(xdg_home).join("hunspell"));
    } else if let Some(home) = &env.home {
        dirs.push(Path::new(home).join(".local/share/hunspell"));
    }
    if let Some(xdg_dirs) = &env.xdg_data_dirs {
        dirs.extend(
            xdg_dirs
                .split(':')
                .filter(|part| !part.is_empty())
                .map(|part| Path::new(part).join("hunspell")),
        );
    }
    dirs.push(PathBuf::from("/usr/local/share/hunspell"));
    dirs.push(PathBuf::from("/usr/share/hunspell"));
    dirs.push(PathBuf::from("/usr/share/myspell/dicts"));
    dirs.push(PathBuf::from("/usr/share/myspell"));
    dirs
}

pub fn discover_dictionaries(env: &EnvSnapshot) -> Vec<SystemDictionary> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for dir in build_search_path(env) {
        scan_dir(&dir, &mut seen, &mut out);
    }
    out
}

fn scan_dir(dir: &Path, seen: &mut HashSet<String>, out: &mut Vec<SystemDictionary>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("dic") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if stem.is_empty() {
            continue;
        }
        let aff_path = dir.join(format!("{stem}.aff"));
        if !aff_path.is_file() {
            continue;
        }
        let tag = canonicalize_tag(stem);
        if !seen.insert(tag.clone()) {
            continue;
        }
        out.push(SystemDictionary {
            tag,
            aff_path,
            dic_path: path,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalize_tag_collapses_case_and_separator_variants() {
        assert_eq!("en-us", canonicalize_tag("en_US"));
        assert_eq!("en-us", canonicalize_tag("EN-us"));
        assert_eq!("pt-br", canonicalize_tag("PT_BR"));
    }

    #[test]
    fn build_search_path_hunspell_dict_dir_is_honoured_first() {
        let dirs = build_search_path(&EnvSnapshot {
            hunspell_dict_dir: Some("/tmp/a:/tmp/b".to_owned()),
            ..EnvSnapshot::default()
        });
        assert_eq!(PathBuf::from("/tmp/a"), dirs[0]);
        assert_eq!(PathBuf::from("/tmp/b"), dirs[1]);
        assert!(
            dirs.iter()
                .any(|dir| dir == Path::new("/usr/share/hunspell"))
        );
    }

    #[test]
    fn build_search_path_xdg_data_home_wins_over_home() {
        let dirs = build_search_path(&EnvSnapshot {
            xdg_data_home: Some("/x/data".to_owned()),
            home: Some("/home/user".to_owned()),
            ..EnvSnapshot::default()
        });
        assert!(dirs.iter().any(|dir| dir == Path::new("/x/data/hunspell")));
        assert!(
            !dirs
                .iter()
                .any(|dir| dir == Path::new("/home/user/.local/share/hunspell"))
        );
    }

    #[test]
    fn discover_dictionaries_picks_up_aff_dic_pairs_and_first_tag_wins() {
        let dir_a = tempfile::tempdir().unwrap();
        let dir_b = tempfile::tempdir().unwrap();
        std::fs::write(dir_a.path().join("en_US.aff"), "").unwrap();
        std::fs::write(dir_a.path().join("en_US.dic"), "").unwrap();
        std::fs::write(dir_b.path().join("EN-us.aff"), "").unwrap();
        std::fs::write(dir_b.path().join("EN-us.dic"), "").unwrap();
        std::fs::write(dir_b.path().join("orphan.dic"), "").unwrap();

        let dicts = discover_dictionaries(&EnvSnapshot {
            hunspell_dict_dir: Some(format!(
                "{}:{}",
                dir_a.path().display(),
                dir_b.path().display()
            )),
            ..EnvSnapshot::default()
        });
        assert_eq!(1, dicts.iter().filter(|dict| dict.tag == "en-us").count());
        assert!(!dicts.iter().any(|dict| dict.tag == "orphan"));
        assert!(
            dicts
                .iter()
                .any(|dict| dict.aff_path == dir_a.path().join("en_US.aff"))
        );
    }
}
