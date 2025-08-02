use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("object parsing error: {}", source))]
    Object { source: object::Error },
    #[snafu(display("windows api error: {}", source))]
    Windows { source: windows::core::Error },
    #[snafu(display("when {}, calls to windows api {} failed: {}", op, call, source))]
    WindowsDetailed {
        source: windows::core::Error,
        call: String,
        op: String,
    },
    #[snafu(display("invalid image file machine: {:?}", machine))]
    InvalidImageFileMachine { machine: u16 },
    #[snafu(context(false), transparent)]
    IO { source: std::io::Error },
    #[snafu(display("{}", msg))]
    Empty { msg: String },
}

pub type Result<T> = std::result::Result<T, Error>;
