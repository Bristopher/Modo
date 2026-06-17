// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ffi::{CStr, CString};
use std::fs::File;
use std::path::Path;
use std::ptr;
use std::sync::{Mutex, MutexGuard};

use fluxer_desktop_native::system_hunspell::dictionaries::{EnvSnapshot, discover_dictionaries};
use fluxer_desktop_native::system_hunspell::encoding::is_utf8_encoding;
use fluxer_desktop_native::system_hunspell::hashing::hash_file_to_hex;
use napi::Task;
use napi::bindgen_prelude::{AsyncTask, Env, Error, Result, Status};
use napi_derive::napi;

static HUNSPELL_LOCK: Mutex<()> = Mutex::new(());

#[napi(object)]
pub struct SystemDictionary {
    pub tag: String,
    #[napi(js_name = "affPath")]
    pub aff_path: String,
    #[napi(js_name = "dicPath")]
    pub dic_path: String,
}

#[napi]
pub struct Hunspell {
    dict: Option<Dictionary>,
}

#[napi]
impl Hunspell {
    #[napi(constructor)]
    pub fn new(aff_path: String, dic_path: String) -> Result<Self> {
        Ok(Self {
            dict: Some(Dictionary::load(&aff_path, &dic_path)?),
        })
    }

    #[napi]
    pub fn spell(&self, word: String) -> Result<bool> {
        validate_text_arg(&word, "word")?;
        Ok(self
            .dict
            .as_ref()
            .is_some_and(|dict| dict.spell(&word).unwrap_or(true)))
    }

    #[napi]
    pub fn suggest(&self, word: String, max: Option<u32>) -> Result<Vec<String>> {
        validate_text_arg(&word, "word")?;
        let Some(dict) = &self.dict else {
            return Ok(Vec::new());
        };
        let limit = normalize_suggestion_limit(max);
        dict.suggest(&word, limit)
    }

    #[napi]
    pub fn add(&mut self, word: String) -> Result<()> {
        validate_text_arg(&word, "word")?;
        if let Some(dict) = &mut self.dict {
            let _ = dict.add(&word);
        }
        Ok(())
    }

    #[napi]
    pub fn remove(&mut self, word: String) -> Result<()> {
        validate_text_arg(&word, "word")?;
        if let Some(dict) = &mut self.dict {
            let _ = dict.remove(&word);
        }
        Ok(())
    }

    #[napi]
    pub fn close(&mut self) {
        self.dict = None;
    }
}

#[napi(js_name = "discoverSystemDictionaries")]
pub fn discover_system_dictionaries() -> Vec<SystemDictionary> {
    let snapshot = EnvSnapshot {
        hunspell_dict_dir: std::env::var("HUNSPELL_DICT_DIR").ok(),
        xdg_data_home: std::env::var("XDG_DATA_HOME").ok(),
        home: std::env::var("HOME").ok(),
        xdg_data_dirs: std::env::var("XDG_DATA_DIRS").ok(),
    };
    discover_dictionaries(&snapshot)
        .into_iter()
        .map(|dict| SystemDictionary {
            tag: dict.tag,
            aff_path: dict.aff_path.display().to_string(),
            dic_path: dict.dic_path.display().to_string(),
        })
        .collect()
}

pub struct HashFileTask {
    path: String,
}

#[napi(js_name = "hashFile")]
pub fn hash_file(path: String) -> Result<AsyncTask<HashFileTask>> {
    validate_path_arg(&path)?;
    Ok(AsyncTask::new(HashFileTask { path }))
}

impl Task for HashFileTask {
    type Output = String;
    type JsValue = String;

    fn compute(&mut self) -> Result<Self::Output> {
        hash_file_to_hex(&self.path).map_err(hash_error)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

#[derive(Debug)]
struct Dictionary {
    handle: *mut hunspell_sys::Hunhandle,
}

impl Dictionary {
    fn load(aff_path: &str, dic_path: &str) -> Result<Self> {
        validate_path_arg(aff_path)?;
        validate_path_arg(dic_path)?;
        if File::open(Path::new(aff_path)).is_err() || File::open(Path::new(dic_path)).is_err() {
            return Err(hunspell_load_error("LoadFailed"));
        }

        let aff = cstring_arg(aff_path, "affPath")?;
        let dic = cstring_arg(dic_path, "dicPath")?;
        let _guard = hunspell_lock();

        let handle = unsafe { hunspell_sys::Hunspell_create(aff.as_ptr(), dic.as_ptr()) };
        if handle.is_null() {
            return Err(hunspell_load_error("LoadFailed"));
        }

        let encoding_ptr = unsafe { hunspell_sys::Hunspell_get_dic_encoding(handle) };
        if encoding_ptr.is_null() {
            unsafe { hunspell_sys::Hunspell_destroy(handle) };
            return Err(hunspell_load_error("LoadFailed"));
        }

        let encoding = unsafe { CStr::from_ptr(encoding_ptr) }
            .to_str()
            .map_err(|_| hunspell_load_error("DictionaryNotUtf8"))?;
        if !is_utf8_encoding(encoding) {
            unsafe { hunspell_sys::Hunspell_destroy(handle) };
            return Err(hunspell_load_error("DictionaryNotUtf8"));
        }

        Ok(Self { handle })
    }

    fn spell(&self, word: &str) -> Result<bool> {
        let word = cstring_arg(word, "word")?;
        let _guard = hunspell_lock();

        Ok(unsafe { hunspell_sys::Hunspell_spell(self.handle, word.as_ptr()) } != 0)
    }

    fn suggest(&self, word: &str, max: usize) -> Result<Vec<String>> {
        let word = cstring_arg(word, "word")?;
        let mut raw: *mut *mut std::os::raw::c_char = ptr::null_mut();
        let _hunspell_guard = hunspell_lock();

        let count = unsafe { hunspell_sys::Hunspell_suggest(self.handle, &mut raw, word.as_ptr()) };
        let _guard = SuggestionList {
            handle: self.handle,
            raw,
            count,
        };
        if count <= 0 || raw.is_null() {
            return Ok(Vec::new());
        }
        let limit = (count as usize).min(max);
        let mut out = Vec::with_capacity(limit);
        for index in 0..limit {
            let item = unsafe { *raw.add(index) };
            if item.is_null() {
                out.push(String::new());
                continue;
            }

            let suggestion = unsafe { CStr::from_ptr(item) };
            if let Ok(text) = suggestion.to_str() {
                out.push(text.to_owned());
            }
        }
        Ok(out)
    }

    fn add(&mut self, word: &str) -> Result<()> {
        let word = cstring_arg(word, "word")?;
        let _guard = hunspell_lock();

        let rc = unsafe { hunspell_sys::Hunspell_add(self.handle, word.as_ptr()) };
        if rc == 0 {
            Ok(())
        } else {
            Err(hunspell_load_error("LoadFailed"))
        }
    }

    fn remove(&mut self, word: &str) -> Result<()> {
        let word = cstring_arg(word, "word")?;
        let _guard = hunspell_lock();

        unsafe { hunspell_sys::Hunspell_remove(self.handle, word.as_ptr()) };
        Ok(())
    }
}

impl Drop for Dictionary {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            let _guard = hunspell_lock();

            unsafe { hunspell_sys::Hunspell_destroy(self.handle) };
            self.handle = ptr::null_mut();
        }
    }
}

struct SuggestionList {
    handle: *mut hunspell_sys::Hunhandle,
    raw: *mut *mut std::os::raw::c_char,
    count: i32,
}

impl Drop for SuggestionList {
    fn drop(&mut self) {
        if self.count > 0 && !self.raw.is_null() {
            unsafe { hunspell_sys::Hunspell_free_list(self.handle, &mut self.raw, self.count) };
        }
    }
}

fn normalize_suggestion_limit(max: Option<u32>) -> usize {
    match max {
        Some(value @ 1..=64) => value as usize,
        _ => 8,
    }
}

fn hunspell_lock() -> MutexGuard<'static, ()> {
    HUNSPELL_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn validate_path_arg(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(Error::new(Status::InvalidArg, "path must be non-empty"));
    }
    if path.as_bytes().contains(&0) {
        return Err(Error::new(
            Status::InvalidArg,
            "path must not contain NUL bytes",
        ));
    }
    Ok(())
}

fn validate_text_arg(value: &str, label: &str) -> Result<()> {
    if value.as_bytes().contains(&0) {
        return Err(Error::new(
            Status::InvalidArg,
            format!("{label} must not contain NUL bytes"),
        ));
    }
    Ok(())
}

fn cstring_arg(value: &str, label: &str) -> Result<CString> {
    CString::new(value).map_err(|_| {
        Error::new(
            Status::InvalidArg,
            format!("{label} must not contain NUL bytes"),
        )
    })
}

fn hunspell_load_error(reason: &str) -> Error {
    Error::new(
        Status::GenericFailure,
        format!("Hunspell load failed: {reason}"),
    )
}

fn hash_error(error: std::io::Error) -> Error {
    let message = match error.kind() {
        std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied => {
            "could not open file for hashing"
        }
        _ => "read failed while hashing file",
    };
    Error::new(Status::GenericFailure, message)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn suggestion_limit_defaults_and_bounds_match_legacy_contract() {
        assert_eq!(8, normalize_suggestion_limit(None));
        assert_eq!(8, normalize_suggestion_limit(Some(0)));
        assert_eq!(1, normalize_suggestion_limit(Some(1)));
        assert_eq!(64, normalize_suggestion_limit(Some(64)));
        assert_eq!(8, normalize_suggestion_limit(Some(65)));
    }

    #[test]
    fn path_validation_rejects_empty_and_nul_paths() {
        assert_eq!(
            "path must be non-empty",
            validate_path_arg("")
                .expect_err("empty path should fail")
                .reason
        );
        assert_eq!(
            "path must not contain NUL bytes",
            validate_path_arg("/tmp/a\0b")
                .expect_err("NUL path should fail")
                .reason
        );
    }

    #[test]
    fn text_validation_rejects_nul_words() {
        assert_eq!(
            "word must not contain NUL bytes",
            validate_text_arg("a\0b", "word")
                .expect_err("NUL word should fail")
                .reason
        );
    }

    #[test]
    fn hash_error_maps_open_failures_to_existing_js_message() {
        let err = hash_file_to_hex("/nonexistent/path/that/should/not/exist.bin")
            .map_err(hash_error)
            .expect_err("missing file should fail");
        assert_eq!(Status::GenericFailure, err.status);
        assert_eq!("could not open file for hashing", err.reason);
    }

    #[test]
    fn hash_file_task_streams_file_to_expected_hex() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("payload");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"abc").unwrap();

        let mut task = HashFileTask {
            path: path.display().to_string(),
        };
        assert_eq!(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            task.compute().unwrap()
        );
    }

    #[test]
    fn dictionary_fixture_spells_and_accepts_runtime_words() {
        let (dir, aff_path, dic_path) = write_fixture_dictionary("UTF-8");
        let mut dict =
            Dictionary::load(&aff_path, &dic_path).expect("fixture dictionary should load");

        assert!(dict.spell("cat").unwrap());
        assert!(dict.spell("cats").unwrap());
        assert!(!dict.spell("Fluxer").unwrap());
        dict.add("Fluxer").unwrap();
        assert!(dict.spell("Fluxer").unwrap());
        dict.remove("Fluxer").unwrap();
        assert!(!dict.spell("Fluxer").unwrap());

        drop(dir);
    }

    #[test]
    fn dictionary_rejects_non_utf8_dictionaries() {
        let (_dir, aff_path, dic_path) = write_fixture_dictionary("ISO-8859-1");
        let err = Dictionary::load(&aff_path, &dic_path)
            .expect_err("non-UTF-8 dictionary should be rejected");
        assert_eq!(Status::GenericFailure, err.status);
        assert_eq!("Hunspell load failed: DictionaryNotUtf8", err.reason);
    }

    fn write_fixture_dictionary(encoding: &str) -> (tempfile::TempDir, String, String) {
        let dir = tempfile::tempdir().unwrap();
        let aff_path = dir.path().join("fixture.aff");
        let dic_path = dir.path().join("fixture.dic");
        std::fs::write(
            &aff_path,
            format!("SET {encoding}\n\nSFX S Y 1\nSFX S 0 s [^sxzhy]\n"),
        )
        .unwrap();
        std::fs::write(&dic_path, "2\ncat/S\nprogram/S\n").unwrap();
        (
            dir,
            aff_path.display().to_string(),
            dic_path.display().to_string(),
        )
    }
}
