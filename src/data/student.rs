use sqlx::{Pool, Postgres};
use serde::Deserialize;
use crate::data::connections::{json_api_get, json_api_get_single};
use crate::data::config::Config;
use crate::macros::err;

#[derive(Deserialize)]
pub struct Student {
    pub id : i32,
    pub name : String,
    pub enrollments : Option<Vec<Enrollment>>,
}

#[derive(Deserialize)]
pub struct Enrollment {
    pub grades : Grades
}

#[derive(Deserialize)]
pub struct Grades {
    pub current_grade : Option<String>,
    pub current_score : Option<f32>
}

impl Student {
    pub async fn load(database : &Pool<Postgres>, config : &Config, course_id : i32) -> Result<(),String> {
        let students = json_api_get::<Student>(config,&format!(
                            "/api/v1/courses/{}/users\
                            ?enrollment_type[]=student\
                            &include[]=total_scores\
                            &include[]=enrollments\
                            &enrollment_state[]=active\
                            &enrollment_state[]=invited\
                            &enrollment_state[]=completed", course_id))
                            .await?;
        let test_student = json_api_get_single::<Student>(config,&format!(
            "/api/v1/courses/{}/student_view_student\
            ?", course_id))
            .await?;
        for s in students.iter() {
            if s.id == test_student.id {
                continue;
            }
            let mut grade = String::new();
            let mut score = 0.0;
            if let Some(enrollments) = &s.enrollments {
                if let Some(enrollment) = enrollments.first() {                   grade = enrollment.grades.current_grade.clone().unwrap_or(String::new());
                    score = enrollment.grades.current_score.unwrap_or(0.0);
                }
            }
            sqlx::query(
                "
                INSERT INTO curr_students 
                (id, course_id, name, curr_grade, curr_score)
                    VALUES
                ($1, $2, $3, $4, $5);
            ")
            .bind(s.id)
            .bind(course_id)
            .bind(s.name.clone())
            .bind(grade)
            .bind(score)
            .execute(database)
            .await
            .map_err(|e| err!("Student SQL Failure", e))?;
        }
        Ok(())
    }

    pub async fn create_table(database : &Pool<Postgres>) -> Result<(), String> {
        sqlx::query(
         "
             CREATE TABLE curr_students(
                 id INT,
                 course_id INT,
                 name TEXT,
                 curr_grade TEXT,
                 curr_score REAL
             );
         ")
         .execute(database)
         .await
         .map_err(|e| err!("SQL Student Table Creation Failure",e))?;
         Ok(())
     }

     pub async fn drop_table(database : &Pool<Postgres>) -> Result<(), String> {
        sqlx::query("DROP TABLE IF EXISTS curr_students;")
            .execute(database)
            .await
            .map_err(|e| err!("SQL Student Table Drop Failure",e))?;
        Ok(())
    }

}
