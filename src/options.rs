use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Options {
    /// Directory to backup
    #[structopt(parse(from_os_str))]
    pub path: std::path::PathBuf
}
