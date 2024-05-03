use clap::Parser;
use qrrs::qrcode::QrCodeViewArguments;
use std::path::PathBuf;
#[derive(Parser)]
#[command(version, about, long_about = None)]

pub struct Cli {
    /// Path to the file or folder to be shared
    pub path: Option<PathBuf>,

    /// Service Port. Default 9527
    #[arg(short, long, default_value = "9527")]
    pub port: u16,

    /// Margin applied to qrcode
    #[arg(name = "margin", long, short = 'm', default_value_t = 5)]
    pub margin: u32,

    /// Invert qrcode colors
    #[arg(name = "invert_colors", long, short = 'i')]
    pub invert_colors: bool,
}

impl From<&Cli> for QrCodeViewArguments {
    fn from(args: &Cli) -> Self {
        Self {
            margin: args.margin,
            invert_colors: args.invert_colors,
        }
    }
}
