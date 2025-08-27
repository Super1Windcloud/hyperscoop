use std::env;

fn get_gitee_env_path() -> String {
    let cwd = env::current_dir().unwrap();
    let env = cwd.join(".env");
    if !env.exists() {
        eprintln!("{}", "当前目录下不存在.env".to_string());
        return "".to_string();
    }
    let env = std::fs::read_to_string(&env).unwrap();
    env.trim().to_string()
}

fn get_github_token_path() -> String {
    let cwd = env::current_dir().unwrap();
    let env = cwd.join(".github_token");
    if !env.exists() {
        eprintln!("{}", "当前目录下不存在.github_token".to_string());
        return "".to_string();
    }
    let env = std::fs::read_to_string(&env).unwrap();
    env.trim().to_string()
}

fn main() {
    let lang = env::var("LANG")
        .or(env::var("LC_ALL"))
        .or(env::var("LC_CTYPE"))
        .unwrap_or_default();
    let lang_prefix = lang.split("_").next().unwrap_or("en");
    let gitee_env = get_gitee_env_path();
    let github_token = get_github_token_path();
    println!("cargo:rustc-env=BUILD_SYSTEM_LANG={}", lang_prefix);
    println!("cargo:rustc-env=GITEE_TOKEN={}", gitee_env);
    println!("cargo:rustc-env=GITHUB_TOKEN={}", github_token);
    if !github_token.is_empty() && !gitee_env.is_empty() {
        println!("cargo:rustc-cfg=token_local");
    } else {
        println!("cargo:rustc-cfg=token_cloud");
    }
    if lang_prefix == "zh" {
        println!("cargo:rustc-cfg=system_lang_zh");
    }
}
