#[derive(clap::Parser, Debug)]
pub struct Config {
    #[clap(long, env)]
    pub server_port: u16,
    #[clap(long, env)]
    pub host: String,
    #[clap(long, env)]
    pub gateway_url: String,
    #[clap(long, env)]
    pub new_room_secret: String,
    #[clap(long, env)]
    pub redis_url: String,
    #[clap(long, env)]
    pub redis_port: u16,
}
