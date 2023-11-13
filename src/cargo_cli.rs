use clap::Parser;
use std::fmt;
use std::str::FromStr;

#[derive(Parser)] // requires `derive` feature
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
pub enum CargoCli {
    Mate(MateArgs),
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
pub struct MateArgs {
    #[arg(long)]
    pub manifest_path: Option<std::path::PathBuf>,
    #[clap(long, default_values_t = vec![Category::Parallel], ignore_case = true)]
    pub category: Vec<Category>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Category {
    Parallel,
    Rules,
    Safety,
}

impl FromStr for Category {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_ref() {
            "parallel" => Ok(Category::Parallel),
            "rules" => Ok(Category::Rules),
            "safety" => Ok(Category::Safety),
            _ => Err("no match"),
        }
    }
}

impl Default for Category {
    fn default() -> Self {
        Category::Parallel
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Category::Parallel => "Parallel",
                Category::Rules => "Rules",
                Category::Safety => "Safety",
            }
        )
    }
}
