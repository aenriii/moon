use clap::Parser;

use super::Subcommand;

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

pub(super) mod up {
    use clap::Parser;

    #[derive(Debug, Parser)]
    pub struct Args {
        #[arg(short, long, default_value = "stable", value_parser(["stable", "nightly", "git"]))]
        pub channel: String,
        #[arg(short, long, default_value = "stable", value_parser(["stable", "ptb", "canary", "development"]))]
        pub branch: String,
    }
}

pub(super) mod down {
    use clap::Parser;

    #[derive(Debug, Parser)]
    pub struct Args {
        #[arg(short, long, default_value = "stable", value_parser(["stable", "ptb", "canary", "development"]))]
        pub branch: String,
    }
}

pub(super) mod dev {
    use clap::Parser;

    #[derive(Debug, Parser)]
    pub struct Args {
        #[arg(short, long, default_value = "stable", value_parser(["stable", "ptb", "canary", "development"]))]
        pub branch: String,
        #[arg(index = 1, value_name = "FOLDER", default_value = "./")]
        pub folder: String,
    }
}

pub(super) mod openasar {
    use clap::Parser;

    #[derive(Debug, Parser)]
    pub struct Args {
        #[arg(short, long, default_value = "stable", value_parser(["stable", "ptb", "canary", "development"]))]
        pub branch: String,

        #[arg(index = 1, value_parser(["up","down"]), value_name = "TOGGLE")]
        pub toggle: String,
    }
}
