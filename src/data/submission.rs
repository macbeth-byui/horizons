use sqlx::{Pool, Postgres};
use serde::Deserialize;
use crate::data::connections::json_api_get;
use crate::data::config::Config;
use crate::macros::err;

#[derive(Deserialize, Debug)]
pub struct Submission {
    pub id : i32,
    pub assignment_id : i32,
    pub user_id : i32,
    pub score : Option<f32>,
    pub excused : Option<bool>,
    pub missing : Option<bool>,
    pub late : Option<bool>,
    pub attempt : Option<i32>,
    pub grade_matches_current_submission : Option<bool>,
}

impl Submission {
    pub async fn load(database : &Pool<Postgres>, config : &Config, course : i32) -> Result<(), String> {
        let submissions = json_api_get::<Submission>(config,&format!(
                            "/api/v1/courses/{}/students/submissions\
                            ?student_ids[]=all\
                            &enrollment_state=active", course))
                            .await?;        
        for s in submissions.iter() {
            sqlx::query(
                "
                INSERT INTO curr_submissions 
                (id, assignment_id, user_id, score, excused,
                 missing, late, attempt, current_submission)
                 VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9);
            ")
            .bind(s.id)
            .bind(s.assignment_id)
            .bind(s.user_id)
            .bind(s.score)
            .bind(s.excused.unwrap_or(false))
            .bind(s.missing.unwrap_or(false))
            .bind(s.late.unwrap_or(false))
            .bind(s.attempt.unwrap_or(0))
            .bind(s.grade_matches_current_submission.unwrap_or(false))
            .execute(database)
            .await
            .map_err(|e| err!("Submission SQL Failure", e))?;
        }
        Ok(())
    }

    
    pub async fn create_table(database : &Pool<Postgres>) -> Result<(), String> {
        sqlx::query(
         "
             CREATE TABLE curr_submissions(
                 id INT,
                 assignment_id INT,
                 user_id INT,
                 score REAL,
                 excused BOOL,
                 missing BOOL,
                 late BOOL,
                 attempt INT,
                 current_submission BOOL);
             ")
         .execute(database)
         .await
         .map_err(|e| err!("SQL Submission Table Creation Failure",e))?;
         Ok(())
    }

    pub async fn drop_table(database : &Pool<Postgres>) -> Result<(), String> {
        sqlx::query("DROP TABLE IF EXISTS curr_submissions;")
            .execute(database)
            .await
            .map_err(|e| err!("SQL Submission Table Drop Failure",e))?;
        Ok(())
    }

}