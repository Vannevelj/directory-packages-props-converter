use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Options {
    /// Directory to convert
    #[structopt(parse(from_os_str))]
    pub path: std::path::PathBuf,

    /// AWS region (backup)
    #[structopt(default_value = "info", short, long)]
    pub log_level: String,
}
