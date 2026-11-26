use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};
use uuid::Uuid;

const SETTINGS_VERSION: u8 = 1;
const MAX_SETTINGS_BYTES: u64 = 8192;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Appearance {
    pub theme: Theme,
    pub text_scale: u8,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            text_scale: 100,
        }
    }
}

impl Appearance {
    pub fn is_valid(self) -> bool {
        matches!(self.text_scale, 100 | 112 | 125)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    schema_version: u8,
    pub onboarding_complete: bool,
    pub appearance: Appearance,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_VERSION,
            onboarding_complete: false,
            appearance: Appearance::default(),
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreferencesUpdate {
    pub appearance: Appearance,
    pub complete_onboarding: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RecoveryNotice {
    InvalidSettings,
    UnreadableSettings,
}

pub struct SettingsRepository {
    directory: PathBuf,
}

impl SettingsRepository {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub fn load(&self) -> (Preferences, Option<RecoveryNotice>) {
        match self.read() {
            Ok(preferences) => (preferences, None),
            Err(error) if error.kind() == io::ErrorKind::NotFound => (Preferences::default(), None),
            Err(error) if error.kind() == io::ErrorKind::InvalidData => (
                Preferences::default(),
                Some(RecoveryNotice::InvalidSettings),
            ),
            Err(_) => (
                Preferences::default(),
                Some(RecoveryNotice::UnreadableSettings),
            ),
        }
    }

    fn read(&self) -> io::Result<Preferences> {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW)
            .open(self.path())?;
        if !file.metadata()?.is_file() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let mut data = Vec::new();
        file.take(MAX_SETTINGS_BYTES + 1).read_to_end(&mut data)?;
        if data.len() as u64 > MAX_SETTINGS_BYTES {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let preferences: Preferences = serde_json::from_slice(&data)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        if preferences.schema_version != SETTINGS_VERSION || !preferences.appearance.is_valid() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        Ok(preferences)
    }

    pub fn save(&self, preferences: Preferences) -> io::Result<()> {
        fs::create_dir_all(&self.directory)?;
        let temporary = self
            .directory
            .join(format!(".settings-{}.tmp", Uuid::new_v4()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)?;
        let result = Self::write_and_replace(&mut file, &temporary, &self.path(), preferences);
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result
    }

    fn write_and_replace(
        file: &mut File,
        temporary: &Path,
        destination: &Path,
        preferences: Preferences,
    ) -> io::Result<()> {
        serde_json::to_writer_pretty(&mut *file, &preferences)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(temporary, destination)
    }

    fn path(&self) -> PathBuf {
        self.directory.join("settings.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("shadow-mind-settings-{}", Uuid::new_v4()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn preferences_round_trip_and_replace_with_private_permissions() {
        let directory = TestDirectory::new();
        let repository = SettingsRepository::new(directory.0.clone());
        assert_eq!(repository.load(), (Preferences::default(), None));
        let mut preferences = Preferences {
            onboarding_complete: true,
            ..Preferences::default()
        };
        repository.save(preferences).unwrap();
        preferences.appearance.theme = Theme::Dark;
        repository.save(preferences).unwrap();
        assert_eq!(repository.load(), (preferences, None));
        assert_eq!(
            fs::metadata(repository.path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
    }

    #[test]
    fn corrupt_oversize_future_and_policy_fields_use_safe_defaults() {
        let directory = TestDirectory::new();
        let repository = SettingsRepository::new(directory.0.clone());
        for invalid in [
            b"{".to_vec(),
            vec![b' '; MAX_SETTINGS_BYTES as usize + 1],
            br#"{"schemaVersion":2,"onboardingComplete":true,"appearance":{"theme":"system","textScale":100}}"#.to_vec(),
            br#"{"schemaVersion":1,"onboardingComplete":true,"appearance":{"theme":"system","textScale":100},"remoteAllowed":true}"#.to_vec(),
            br#"{"schemaVersion":1,"onboardingComplete":true,"appearance":{"theme":"system","textScale":10}}"#.to_vec(),
        ] {
            fs::write(repository.path(), &invalid).unwrap();
            assert_eq!(
                repository.load(),
                (Preferences::default(), Some(RecoveryNotice::InvalidSettings))
            );
            assert_eq!(fs::read(repository.path()).unwrap(), invalid);
        }
    }

    #[test]
    fn failed_replace_removes_temporary_file() {
        let directory = TestDirectory::new();
        let repository = SettingsRepository::new(directory.0.clone());
        fs::create_dir(repository.path()).unwrap();
        assert!(repository.save(Preferences::default()).is_err());
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
        assert!(repository.path().is_dir());
    }
}
