use terman_tmux::run_with_binary_parse;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_with_binary_parse()
}