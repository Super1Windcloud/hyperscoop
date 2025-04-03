use bat::PrettyPrinter;

pub fn catch_manifest(bucket_paths: Vec<String>, app_name: String) -> anyhow::Result<()> {
    for bucket_path in bucket_paths.iter() {
        let manifest_path = bucket_path.clone() + "\\bucket";
        for file in std::fs::read_dir(manifest_path)? {
            let entry = file?;
            let file_type = entry.file_type()?;
            let file = entry.path();
            let file_str = file.as_path().display().to_string();
            if file_type.is_file() && file_str.ends_with(".json") {
                let file_name = file.file_stem().unwrap().to_str().unwrap();
                if file_name != app_name {
                    continue;
                }
                let content = std::fs::read_to_string(file)?;
                let buffer = content.as_bytes();
                PrettyPrinter::new()
                    .input_from_bytes(buffer)
                    .language("json")
                    .print()?;
                   return  Ok(()); 
            }
        }
    }
    Ok(())
}
