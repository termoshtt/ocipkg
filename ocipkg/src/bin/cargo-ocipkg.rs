use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "cargo-ocipkg")]
enum Opt {
    /// Build library or executable, and pack as a container
    Build {
        #[structopt(long)]
        release: bool,
    },

    /// Push container to OCI registry
    Publish {},
}

fn main() {
    match Opt::from_args() {
        Opt::Build { release } => {
            dbg!(release);
            todo!("cargo-ocipkg build")
        }
        Opt::Publish {} => {
            todo!("cargo-ocipkg publish")
        }
    }
}
