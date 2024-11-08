use serde::{Deserialize, Serialize};


#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub struct SearchManifest {
  //Serde 会自动忽略 JSON 文件中不存在的字段或者在结构体中没有定义的字段。
  #[serde(skip_serializing_if = "Option::is_none")] // 只序列化非空值
  pub version: Option<String>,

  // pub description: Option<String>,

  //  主页 URL
  // pub homepage: Option<String>,
}


impl SearchManifest {
  #[must_use]
  #[inline]  // 内联函数，编译器可以优化代码，提高运行效率
  pub fn get_version(&self) -> Option<&str> {
    self.version.as_deref()
  }

  pub fn set_version(&mut self, version: String) {
    self.version = Some(version);
  }
}
