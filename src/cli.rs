    use crate::stellarhosts;
    use clap::{Parser, Subcommand};

    #[derive(Parser)]
    #[command(version, about, long_about = None)]
    struct Cli {
        #[arg(short, long, action = clap::ArgAction::Count)]
        debug: u8,

        #[command(subcommand)]
        command: Option<Commands>,
    }

    #[derive(Subcommand)]
    enum Commands {
        /// load data from vot files into the storage
        LoadData,
    }

    pub fn run_cli() {
        // no client-side main function
        // unless we want this to work with e.g., Trunk for a purely client-side app
        // see lib.rs for hydration function instead

        use clap::CommandFactory;

        let cli = Cli::parse();

        // TODO: setup logging depending on verbosity level
        match cli.debug {
            0 => println!("Debug mode is off"),
            1 => println!("Debug mode is kind of on"),
            2 => println!("Debug mode is on"),
            _ => println!("Don't be crazy"),
        }

        // You can check for the existence of subcommands, and if found use their
        // matches just as you would the top level cmd
        match &cli.command {
            Some(Commands::LoadData) => {
                stellarhosts::load_data();
            }
            None => {
                let mut cmd = Cli::command();
                let _ = cmd.print_long_help();
            }
        }
    }
}
