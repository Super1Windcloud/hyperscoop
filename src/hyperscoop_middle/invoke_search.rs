use crate::command_args::search::SearchArgs;
use command_util_lib::search::{exact_search, fuzzy_search};

pub fn execute_search_command(query: SearchArgs) -> Result<(), anyhow::Error> {
    // 如果没有  -e 选项，则模糊匹配
    if !query.exact_match_option {
        fuzzy_search(query.name)
    } else {
        exact_search(query.name)
    }
    Ok(())
}
