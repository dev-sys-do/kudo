use log::LevelFilter;

pub struct Config {
    config_file: String,
    constroller_url: String,
    verbosity_level: LevelFilter,
}

pub fn read_env() {
    // TODO
}

pub fn read_config(file: String) -> Config {
    // TODO
}
