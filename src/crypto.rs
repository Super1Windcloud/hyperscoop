#![allow(dead_code, unused_variables, unused_imports)] // all FIle 

use magic_crypt::{new_magic_crypt, MagicCryptTrait};

fn h_t_s(hex: &str) -> Result<String, String> {
    let bytes = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|e| format!("解析十六进制失败: {}", e))?;

    String::from_utf8(bytes).map_err(|e| format!("UTF-8 解码失败: {}", e))
}

pub fn encrypt() -> anyhow::Result<String > {
    let str = "3166626136396461326633346437623062343263363831323135336436643132";
    let mc = new_magic_crypt!("superwindcloud", 256);
    let str = mc.encrypt_str_to_base64(h_t_s(str).unwrap());
    println!("{}", str);
    Ok(str)
}

pub fn decrypt_gitee() -> anyhow::Result<String> {
    let str = "Rkaxww0ahN0jnlvlpED+whcOuUymF2oMLOieNuTnrMi4wVl41uk/HJg2a1xKWG9y";
    let mc = new_magic_crypt!("superwindcloud", 256);
    let str = mc.decrypt_base64_to_string(str).expect("解密失败");
    println!("{}", str);
    Ok(str)
}

 
mod test_crypto {
    use crate::crypto::{decrypt_gitee};
    #[test]
    fn test_encrypt() {
      decrypt_gitee().unwrap();
    }
}
