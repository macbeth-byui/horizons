use serde::Deserialize;
use sqlx::{Pool, Postgres};
use crate::data::connections::json_api_get;
use crate::data::config::Config;
use crate::macros::err;

#[derive(Deserialize)]
pub struct Assignment {
    pub id : i32,
    pub name : String,
    pub quiz_id : Option<u32>,
    pub points_possible : f32,
    pub assignment_group_id : i32
}

impl Assignment {
    pub async fn load(database : &Pool<Postgres>, config : &Config, course_id : i32) -> Result<(), String> {
        let assignments = json_api_get::<Assignment>(config,&format!(
                            "/api/v1/courses/{}/assignments?", course_id))
                            .await?;
        for a in assignments.iter() {
            sqlx::query(
                "
                INSERT INTO curr_assignments 
                (id, course_id, name, points_possible,
                 assignment_group_id)
                    VALUES
                ($1, $2, $3, $4, $5);
            ")
            .bind(a.id)
            .bind(course_id)
            .bind(a.name.clone())
            .bind(a.points_possible)
            .bind(a.assignment_group_id)
            .execute(database)
            .await
            .map_err(|e| err!("Assignment SQL Failure", e))?;
        }
        Ok(())
    }


    pub async fn create_table(database : &Pool<Postgres>) -> Result<(), String> {
        sqlx::query(
         "
             CREATE TABLE curr_assignments(
                 id INT,
                 course_id INT,
                 name TEXT,
                 points_possible INT,
                 assignment_group_id INT
             );
         ")
         .execute(database)
         .await
         .map_err(|e| err!("SQL Assignment Table Creation Failure",e))?;
         Ok(())
     }

     pub async fn drop_table(database : &Pool<Postgres>) -> Result<(), String> {
        sqlx::query("DROP TABLE IF EXISTS curr_assignments;")
            .execute(database)
            .await
            .map_err(|e| err!("SQL Assignment Table Drop Failure",e))?;
        Ok(())
    }


}