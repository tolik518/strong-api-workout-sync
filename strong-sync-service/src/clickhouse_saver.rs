use clickhouse::Row;
use clickhouse::insert::Insert;
use serde::{Deserialize, Serialize};
use std::error::Error;
use strong_api_lib::data_transformer::Workout;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

/// This flattened struct represents one set with its workout and exercise context.
#[derive(Row, Serialize, Deserialize, Debug)]
pub struct WorkoutSet {
    #[serde(with = "clickhouse::serde::uuid")]
    pub workout_id: Uuid,
    pub workout_name: String,
    pub timezone: String,
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub start_date: OffsetDateTime,
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub end_date: OffsetDateTime,
    #[serde(with = "clickhouse::serde::uuid")]
    pub exercise_id: Uuid,
    pub exercise_nr: u32,
    pub exercise_name: String,
    #[serde(with = "clickhouse::serde::uuid")]
    pub set_id: Uuid,
    pub set_nr: u32,
    pub weight: f32,
    pub reps: u32,
    pub rpe: f32,
}

pub struct ClickHouseSaver {
    client: clickhouse::Client,
    table_name: String,
}

impl ClickHouseSaver {
    pub fn new(
        url: &str,
        username: &str,
        password: &str,
        database: &str,
        table_name: &str,
    ) -> Self {
        Self {
            client: clickhouse::Client::default()
                .with_url(url)
                .with_user(username)
                .with_password(password)
                .with_database(database),
            table_name: table_name.to_string(),
        }
    }

    /// Saves a given workout into ClickHouse by flattening its nested data into rows.
    ///
    /// # Arguments
    ///
    /// * `workout` - A reference to the Workout struct.
    ///
    /// # Returns
    ///
    /// A Result indicating success or any error encountered.
    pub async fn save_workout(&self, workout: &Workout) -> Result<(), Box<dyn Error>> {
        let mut insert: Insert<WorkoutSet> = self.client.insert(&self.table_name)?;

        for exercise in &workout.exercises {
            let exercise_nr = workout
                .exercises
                .iter()
                .position(|x| x.id == exercise.id)
                .unwrap() as u32;
            for set in &exercise.sets {
                let start_dt = OffsetDateTime::parse(
                    &workout.start_date.clone().unwrap_or_default(),
                    &Rfc3339,
                )?;

                let end_dt =
                    OffsetDateTime::parse(&workout.end_date.clone().unwrap_or_default(), &Rfc3339)?;

                let set_nr = exercise.sets.iter().position(|x| x.id == set.id).unwrap() as u32;

                let row = WorkoutSet {
                    workout_id: Uuid::parse_str(&workout.id).expect("workout_id UUID parse failed"),
                    workout_name: workout.name.clone(),
                    timezone: workout
                        .timezone
                        .clone()
                        .unwrap_or_else(|| "Europe/Berlin".to_string()),
                    start_date: start_dt,
                    end_date: end_dt,
                    exercise_id: Uuid::parse_str(&exercise.id).expect("exercise UUID parse failed"),
                    exercise_nr,
                    exercise_name: exercise.name.clone(),
                    set_id: Uuid::parse_str(&set.id).expect("set UUID parse failed"),
                    set_nr,
                    weight: set.weight.unwrap_or(0.0),
                    reps: set.reps,
                    rpe: set.rpe.unwrap_or(0.0),
                };
                // debug print set.rpe.unwrap_or(0.0)
                println!("Inserting row: {:?}", row);

                insert.write(&row).await?;
            }
        }

        insert.end().await?;

        println!("Workout {} imported successfully", workout.id);
        Ok(())
    }
}
