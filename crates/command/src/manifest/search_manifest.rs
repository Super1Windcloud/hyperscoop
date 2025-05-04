use serde::{Deserialize, Serialize};


#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub struct SearchManifest {
  #[serde(skip_serializing_if = "Option::is_none")] // 只序列化非空值
  pub version: Option<String>,
 
}


impl SearchManifest {
  #[must_use]
  #[inline]  
  pub fn get_version(&self) -> Option<&str> {
    self.version.as_deref()
  }

  pub fn set_version(&mut self, version: String) {
    self.version = Some(version);
  }
}
