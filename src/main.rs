#[derive(Parser, Debug)]
struct Args {
    /// API key
    #[arg(short = 't', long = "apikey", env = "MLN_API_KEY")]
    apikey: Option<String>,
}

fn main() {}
