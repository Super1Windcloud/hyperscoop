use clap::{ builder::PossibleValue ,  ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CommandOption {
  subcommand,
  unrecognized,
}

// Can also be derived with feature flag `derive`
impl ValueEnum for CommandOption {
  fn value_variants<'a>() -> &'a [Self] {
    &[CommandOption::subcommand, CommandOption::unrecognized]
  }

  fn to_possible_value(&self) -> Option<PossibleValue> {
    Some(match self {
      CommandOption::subcommand => PossibleValue::new("fast").help("Run swiftly"),
      CommandOption::unrecognized => PossibleValue::new("slow").help("Crawl slowly but steadily"),
    })
  }
}

impl std::fmt::Display for CommandOption {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.to_possible_value()
      .expect("no values are skipped")
      .get_name()
      .fmt(f)
  }
}

fn main() {}