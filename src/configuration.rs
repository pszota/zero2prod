//! src/configuration.rs
#[derive(serde::Deserialize)]
pub struct Settings{
    pub database: DatabeseSettings,
    pub application_port: u16
}

#[derive(serde::Deserialize,Clone)]
pub struct DatabeseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings,config::ConfigError> {

let settings = config::Config::builder()
        .add_source (
                config::File::new("configuration.yaml",config::FileFormat::Yaml)
        )
        .build()?;

    settings.try_deserialize::<Settings>()

}

impl DatabeseSettings {
    pub fn connection_string (&self)->String{

        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,self.password,self.host,self.port,self.database_name
        )
    }
}

