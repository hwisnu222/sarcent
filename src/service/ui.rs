use clap::{Parser,Subcommand, Args};

#[derive(Debug, Parser)]
#[command(version, about="sercent - file server worker")]
pub struct Cli{
    #[command(subcommand)]
    pub command: Command
}

#[derive(Subcommand, Debug)]
pub enum Command{
    Master{
        #[arg(short, long, default_value_t=10)]
        interval: u64,

        #[arg(short, long, default_value=".")]
        target: String
    },
    Worker{
        #[arg(short,long, required=true)]
        master_addr: String,

        #[arg(short, long, default_value=".")]
        target: String
    },

    Node(NodeArgs)
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
  List
}
