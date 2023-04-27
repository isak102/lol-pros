use riven::RiotApi;

lazy_static::lazy_static! {
    pub static ref RIOT_API: RiotApi = {
        let api_key = std::env::var("RGAPI_KEY")
            .expect("RGAPI_KEY environment variable not defined.");
        RiotApi::new(api_key)
    };
}
