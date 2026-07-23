use serde::Deserialize;

use super::PluginPresetEntries;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(try_from = "PluginPresetEntries")]
pub struct BabelPresets {
    pub errors: Vec<String>,
    pub unsupported: Vec<String>,

    pub jsx: bool,
    pub typescript: bool,
}

impl TryFrom<PluginPresetEntries> for BabelPresets {
    type Error = String;

    fn try_from(entries: PluginPresetEntries) -> Result<Self, Self::Error> {
        let mut presets = Self::default();
        for entry in entries.0 {
            match entry.normalized_name() {
                "react" => {
                    presets.jsx = true;
                    presets.unsupported.push(entry.name().to_owned());
                }
                "typescript" => {
                    presets.typescript = true;
                    presets.unsupported.push(entry.name().to_owned());
                }
                _ => presets.unsupported.push(entry.name().to_owned()),
            }
        }
        Ok(presets)
    }
}
