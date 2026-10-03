use crate::command_args::link::LinkArgs;
use crate::i18n::tr_fmt;
use command_util_lib::reset::reset_latest_version;
use crossterm::style::Stylize;

pub fn execute_link_command(args: LinkArgs) -> anyhow::Result<()> {
    for app_name in &args.app_names {
        reset_latest_version(app_name, args.global, true)?;
        println!(
            "{}",
            tr_fmt!(
                "Successfully linked '{name}'. Executables are now in PATH.",
                "已成功链接 '{name}'，命令现已加入 PATH 环境。",
                name = app_name
            )
            .green()
            .bold()
        );
    }
    Ok(())
}
