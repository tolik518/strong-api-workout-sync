mod clickhouse_saver;

use dotenvy::dotenv;
use reqwest::Url;
use std::env;
use std::fs;
use std::path::Path;
use strong_api_lib::data_transformer::{DataTransformer, Workout};
use strong_api_lib::models::measurement::MeasurementsResponse;
use strong_api_lib::strong_api::{Includes, StrongApi};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("strong-sync-service v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    dotenv().ok();

    // Load configuration from environment variables.
    let config = load_config()?;
    let url = Url::parse(&config.strong_backend).expect("STRONG_BACKEND is not a valid URL");

    // Initialize the API and ClickHouse saver.
    let mut strong_api = StrongApi::new(url);
    let clickhouse_saver = create_clickhouse_saver(&config);

    // Log in to the API.
    strong_api
        .login(config.username.as_str(), config.password.as_str())
        .await?;

    // Get the measurements (either from file or API).
    let measurements_response = get_measurements_response(&mut strong_api).await?;

    // Fetch user data with logs.
    let user = strong_api.get_user("", 500, vec![Includes::Log]).await?;

    println!(
        "Measurements count: {}/{}",
        measurements_response.embedded.measurements.len(),
        measurements_response.total
    );

    // Transform the measurements into workouts.
    let data_transformer = DataTransformer::new().with_measurements_response(measurements_response);
    let workouts = data_transformer
        .get_measurements_from_logs(&user.embedded.log)
        .expect("Couldn't read workouts");

    println!("Workout count: {}", workouts.len());

    // Save each workout using the ClickHouse saver.
    save_workouts(&workouts, &clickhouse_saver).await?;

    Ok(())
}

/// Holds all configuration values loaded from the environment.
struct Config {
    username: String,
    password: String,
    strong_backend: String,
    clickhouse_url: String,
    clickhouse_user: String,
    clickhouse_pass: String,
    clickhouse_database: String,
    clickhouse_table: String,
}

/// Load configuration values from environment variables.
fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    Ok(Config {
        username: env::var("STRONG_USER").expect("STRONG_USER must be set"),
        password: env::var("STRONG_PASS").expect("STRONG_PASS must be set"),
        strong_backend: env::var("STRONG_BACKEND").expect("STRONG_BACKEND must be set"),
        clickhouse_url: env::var("CLICKHOUSE_URL").expect("CLICKHOUSE_URL must be set"),
        clickhouse_user: env::var("CLICKHOUSE_USER").expect("CLICKHOUSE_USER must be set"),
        clickhouse_pass: env::var("CLICKHOUSE_PASS").expect("CLICKHOUSE_PASS must be set"),
        clickhouse_database: env::var("CLICKHOUSE_DATABASE")
            .expect("CLICKHOUSE_DATABASE must be set"),
        clickhouse_table: env::var("CLICKHOUSE_TABLE").expect("CLICKHOUSE_TABLE must be set"),
    })
}

/// Create a new ClickHouseSaver instance using the provided configuration.
fn create_clickhouse_saver(config: &Config) -> clickhouse_saver::ClickHouseSaver {
    clickhouse_saver::ClickHouseSaver::new(
        config.clickhouse_url.as_str(),
        config.clickhouse_user.as_str(),
        config.clickhouse_pass.as_str(),
        config.clickhouse_database.as_str(),
        config.clickhouse_table.as_str(),
    )
}

/// Retrieve the measurements response either by reading from a file or fetching from the API.
async fn get_measurements_response(
    strong_api: &mut StrongApi,
) -> Result<MeasurementsResponse, Box<dyn std::error::Error>> {
    if !Path::new("measurements.json").exists() {
        println!("Fetching measurements from API");
        let measurements_response_page1 = strong_api.get_measurements(1).await?;
        let measurements_response_page2 = strong_api.get_measurements(2).await?;
        let measurements_response = measurements_response_page1.merge(measurements_response_page2);
        let measurements_json = serde_json::to_string(&measurements_response)?;
        fs::write("measurements.json", measurements_json)?;
        Ok(measurements_response)
    } else {
        println!("Reading measurements from file");
        let measurements_json = fs::read_to_string("measurements.json")?;
        let measurements_response = serde_json::from_str(&measurements_json)?;
        Ok(measurements_response)
    }
}

/// Save all workouts to ClickHouse.
async fn save_workouts(
    workouts: &[Workout],
    clickhouse_saver: &clickhouse_saver::ClickHouseSaver,
) -> Result<(), Box<dyn std::error::Error>> {
    for workout in workouts {
        clickhouse_saver
            .save_workout(workout)
            .await
            .expect("Couldn't save workout");
    }
    Ok(())
}
