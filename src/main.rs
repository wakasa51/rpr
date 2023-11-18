use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Opts {
    #[arg(short, long, value_name = "OWNER", help = "Sets the repository owner")]
    owner: String,

    #[arg(
        short,
        long,
        value_name = "REPO_NAME",
        help = "Sets the repository name"
    )]
    name: String,

    #[arg(short, long, value_name = "COMMIT_HASH", help = "Sets the commit hash")]
    commit: String,

    #[arg(
        short,
        long,
        value_name = "ACCESS_TOKEN",
        help = "Sets the GitHub access token"
    )]
    token: String,
}

fn main() {
    // コマンドライン引数をパース
    let opts: Opts = Opts::parse();

    println!("{}", opts.owner);
    println!("{}", opts.name);
    println!("{}", opts.commit);
}
