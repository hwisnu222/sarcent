use clap::{Parser,Subcommand, Args, ArgAction::SetTrue};

#[derive(Debug, Parser)]
#[command(version, about="sercent - file server worker")]
pub struct Cli{
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, action=SetTrue,)]
    pub verbose: bool,

}

#[derive(Subcommand, Debug)]
pub enum Commands{
    Master{
        #[arg(short, long, default_value=".")]
        source: String,

        // #[arg(short='d', long="detach", action=SetTrue)]
        // detach: bool,

    },
    Worker{
        #[arg(short, long, default_value=".")]
        target: String,

        #[arg(short, long, default_value="50051")]
        port: u16

        // #[arg(short='d', long="detach", action=SetTrue)]
        // detach: bool,
    },

    Node(NodeArgs),

    Search{
        #[arg(short, long)]
        filename: String,
    }
}

#[derive(Args,Debug)]
pub struct NodeArgs{
    #[command(subcommand)]
    pub action: NodeAction

}

#[derive(Subcommand, Debug)]
pub enum NodeAction{
  Add{
    #[arg(long, required=true, value_delimiter=',')]
    ips: Vec<String>
  },
  List,
  Remove{
      #[arg(long, required=true)]
      ip: String
  },
}
