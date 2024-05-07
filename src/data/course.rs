use sqlx::{Pool, Postgres};
use serde::Deserialize;
use crate::data::connections::json_api_get_single;
use crate::data::config::Config;
use crate::macros::err;

#[derive(Deserialize)]
pub struct Course {
    pub id : i32,
    pub course_code : String,
    pub concluded : bool,
    pub term : CourseTerm,
    pub total_students : Option<i32>,
}

#[derive(Deserialize)]
pub struct CourseTerm {
    pub name : String
}

impl Course {
    // pub async fn get_courses(config : &Config, current : bool) -> io::Result<Vec<Self>> {
    //     let mut courses = json_api_get::<Course>(config, 
    //         "/api/v1/courses\
    //                 ?enrollment_type=teacher\
    //                 &state=available\
    //                 &include[]=concluded\
    //                 &include[]=term\
    //                 &include[]=total_students").await?;
    //     courses.iter_mut().for_each(|x| x.term.name = 
    //         Course::convert_term(&x.term.name));
    //     courses.retain(|x| !x.term.name.is_empty());
    //     if current {
    //         courses.retain(|x| !x.concluded);
    //     }
    //     courses.sort_by(|x, y| x.course_code.cmp(&y.course_code));
    //     Ok(courses)
    // }

    pub async fn create_table(database : &Pool<Postgres>) -> Result<(), String> {
       sqlx::query(
        "
            CREATE TABLE curr_courses(
                id INT,
                code TEXT,
                concluded BOOLEAN,
                term TEXT,
                students INT
            );
        ")
        .execute(database)
        .await
        .map_err(|e| err!("SQL Course Table Creation Failure",e))?;
        Ok(())
    }

    pub async fn drop_table(database : &Pool<Postgres>) -> Result<(), String> {
        sqlx::query("DROP TABLE IF EXISTS curr_courses;")
            .execute(database)
            .await
            .map_err(|e| err!("SQL Course Table Drop Failure",e))?;
        Ok(())
    }

    pub async fn load(database : &Pool<Postgres>, config : &Config, course_ids : &[i32]) -> Result<(),String> {
        let mut courses = Vec::<Self>::new();
        for id in course_ids.iter() {
            let course = json_api_get_single(config, 
                &format!("/api/v1/courses/{}\
                                 ?include[]=term\
                                 &include[]=concluded\
                                 &include[]=total_students",id)) 
                                 .await?;
            courses.push(course)
        }

        for c in courses.iter() {
            sqlx::query(
                "
                INSERT INTO curr_courses
                (id, code, concluded, term, students)
                 VALUES
                ($1, $2, $3, $4, $5);
            ")
            .bind(c.id)
            .bind(c.course_code.clone())
            .bind(c.concluded)
            .bind(Course::convert_term(&c.term.name))
            .bind(c.total_students.unwrap_or(0))
            .execute(database)
            .await
            .map_err(|e| err!("Course SQL Failure", e))?;
        }
        Ok(())
    }


    fn convert_term(term : &str) -> String {
        let mut parts = term.split_whitespace();
        let season = parts.next().unwrap_or("");
        let year = parts.next().unwrap_or("");
        let period = match season {
            "Winter" => "1W",
            "Spring" => "2S",
            "Summer" => "3M",
            "Fall" => "4F",
            _ => return String::new()
        };
        format!("{}-{}",year,period)
    }
}