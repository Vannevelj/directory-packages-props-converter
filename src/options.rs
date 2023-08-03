use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Options {
    /// Directory to convert
    #[structopt(parse(from_os_str))]
    pub path: std::path::PathBuf
}
