use chrono::{DateTime, Local, Utc};
use std::fs::metadata;
use std::path::Path;
use std::time::UNIX_EPOCH;

pub fn get_dir_updated_time(dir_path: &Path) -> String {
    let metadata = metadata(&dir_path).expect("Failed to get metadata");
    let modified_time = metadata.modified().expect("Failed to get modified time");
    // 将修改时间转换为自 UNIX_EPOCH 以来的时间戳
    let duration_since_epoch = modified_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let updated_time = UNIX_EPOCH + duration_since_epoch; // 这里得到的是一个`SystemTime`
                                                          // UTC 是全球标准时间
    let updated_time_utc: DateTime<Utc> = updated_time.into(); // 转换为 `DateTime<Utc>`
                                                               // 转换为CST 中国北京时间
    let updated_time_cst = updated_time_utc.with_timezone(&Local);
    let updated_time_formatted = updated_time_cst.format("%Y-%m-%d %H:%M:%S").to_string();

    updated_time_formatted
}

pub fn get_file_updated_time(file_path: &Path) -> String {
    let metadata = metadata(&file_path).expect("Failed to get metadata");
    let modified_time = metadata.modified().expect("Failed to get modified time");
    // 将修改时间转换为自 UNIX_EPOCH 以来的时间戳
    let duration_since_epoch = modified_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let updated_time = UNIX_EPOCH + duration_since_epoch; // 这里得到的是一个`SystemTime`
                                                          // UTC 是全球标准时间
    let updated_time_utc: DateTime<Utc> = updated_time.into(); // 转换为 `DateTime<Utc>`
                                                               // 转换为CST 中国北京时间
    let updated_time_cst = updated_time_utc.with_timezone(&Local);
    let updated_time_formatted = updated_time_cst.format("%Y-%m-%d %H:%M:%S").to_string();

    updated_time_formatted
}
