use std::collections::HashMap;
use tokio::task::JoinHandle;
use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use crate::data::assignment::Assignment;
use crate::data::submission::Submission;
use crate::modules::module::ModuleTrait;
use crate::data::config::Config;
use crate::data::course::Course;
use crate::data::student::Student;
use crate::macros::err;

pub struct CurrentMod {
    config : Config,
    course_lookup : HashMap<String, i32>,
    database : Pool<Postgres>,
}

#[async_trait]
impl ModuleTrait for CurrentMod {
    fn get_name(&self) -> String {
        "Current".to_string()
    }

    async fn process_cmd(&mut self, parsed : Vec<&str>) -> Result<bool,String> {
        if let Some(command) = parsed.first() {
            return match *command {
                "courses" => {
                    self.course_list().await?;
                    Ok(true)
                }
                "students" => {
                    if let Some(course) = parsed.get(1) {
                        if let Some(course_id) = self.course_lookup.get(*course) {
                            self.students(*course_id).await?;
                        }
                        else {
                            println!("Invalid Course ID");
                        }
                    }
                    else {
                        println!("Missing Course ID");
                    }
                    Ok(true)
                }
                "grades" => {
                    if let Some(course) = parsed.get(1) {
                        if let Some(course_id) = self.course_lookup.get(*course) {
                            self.grades(*course_id).await?;
                        }
                        else {
                            println!("Invalid Course ID");
                        }
                    }
                    else {
                        println!("Missing Course ID");
                    }
                    Ok(true)
                }                
                _ => Ok(false)
            }
        }
        Ok(false)
    }

    async fn refresh(&mut self) -> Result<(),String> {
        println!("Loading Module: {}", self.get_name());

        Course::drop_table(&self.database).await?;
        Assignment::drop_table(&self.database).await?;
        Student::drop_table(&self.database).await?;
        Submission::drop_table(&self.database).await?;

        Course::create_table(&self.database).await?;
        Assignment::create_table(&self.database).await?;
        Student::create_table(&self.database).await?;
        Submission::create_table(&self.database).await?;

        let course_ids = self.course_lookup
            .values()
            .map(|x| x.to_owned())
            .collect::<Vec<i32>>();

        Course::load(&self.database, &self.config, &course_ids).await?;

        let mut threads = Vec::<JoinHandle<Result<(),String>>>::new();
        for course in self.config.current_config.courses.iter() {
            {
                let d = self.database.clone();
                let c = self.config.clone();
                let i = course.1;
                let t = tokio::spawn(async move {
                    Student::load(&d, &c, i).await
                });
                threads.push(t);
            }

            {
                let d = self.database.clone();
                let c = self.config.clone();
                let i = course.1;
                let t = tokio::spawn(async move {
                    Assignment::load(&d, &c, i).await
                });
                threads.push(t);
            }

            {
                let d = self.database.clone();
                let c = self.config.clone();
                let i = course.1;
                let t = tokio::spawn(async move {
                    Submission::load(&d, &c, i).await
                });
                threads.push(t);
            }

        }

        for t in threads {
            t.await.unwrap()?;
        }
        println!();

        Ok(())
    }

    fn help(&self) {
        println!("courses");
        println!("students <course id>");
        println!("grades <course id>");
        print!("     <course id> =");
        for course in &self.config.current_config.courses {
            print!(" {}", course.0);
        }
        println!();

    }

}

impl CurrentMod {
    pub fn new(config : Config, database : Pool<Postgres>) -> Self {
        let mut course_lookup = HashMap::<String,i32>::new();
        for course in config.current_config.courses.iter() {
            course_lookup.insert(course.0.clone(), course.1);
        }
        Self {config, course_lookup, database }
    }

    pub async fn course_list(&self) -> Result<(),String> {
        #[derive(sqlx::FromRow)]
        struct Query {
            code : String,
            term : String,
            students : i32
        }

        let results = sqlx::query_as::<_,Query>(
                "
                SELECT code, term, students
                FROM curr_courses
                ORDER BY code;
            ")
            .fetch_all(&self.database)
            .await
            .map_err(|e| err!("Course List SQL Query Failure",e))?;
        println!("{:<12} {:<8} {:<5}",
            "CID","TERM","SIZE");
        println!("{:-<12} {:-<8} {:-<5}",
            "", "", "");
        for result in results {
            println!("{:<12} {:<8} {:<5}",
                result.code, result.term, result.students);
        }
        Ok(())
    }

    async fn students(&self, course : i32) -> Result<(),String> {
        #[derive(sqlx::FromRow)]
        struct Query {
            name : String,
            submitted : i64,
            missing : i64,
            excused : i64,
            ungraded_init : i64,
            ungraded_resubmit : i64,
            curr_grade : String,
            curr_score : f32
        }
        let results = sqlx::query_as::<_,Query>(
                "
                SELECT 
                    stu.name,
                    SUM(CASE WHEN sub.attempt > 0 THEN 1 ELSE 0 END) as submitted,
                    SUM(CASE WHEN sub.missing THEN 1 ELSE 0 END) as missing,
                    SUM(CASE WHEN sub.excused THEN 1 ELSE 0 END) as excused,
                    SUM(CASE WHEN sub.current_submission THEN 0 ELSE 1 END) as something,
                    SUM(CASE WHEN sub.score IS NULL and sub.attempt > 0 THEN 1 ELSE 0 END) as ungraded_init,
                    SUM(CASE WHEN not sub.current_submission THEN 1 ELSE 0 END) as ungraded_resubmit,
                    stu.curr_grade,
                    stu.curr_score
                FROM curr_students AS stu
                INNER JOIN curr_submissions AS sub
                    ON stu.id = sub.user_id
                INNER JOIN curr_assignments as asn
                    ON asn.id = sub.assignment_id
                WHERE asn.course_id = $1 and stu.course_id = $1 and asn.points_possible > 0
                GROUP BY stu.name, stu.curr_grade, stu.curr_score
                ORDER BY stu.curr_score DESC, stu.name ASC;
            ")
            .bind(course)
            .fetch_all(&self.database)
            .await
            .map_err(|e| err!("Course List SQL Query Failure",e))?;
        println!("{:40} {:4} {:4} {:4} {:4} {:4} {:7} {:5}",
            "NAME","SUBM","MISS","EXCU", "UG-I", "UG-R", "SCORE-%", "GRADE"
        );
        println!("{:-<40} {:-<4} {:-<4} {:-<4} {:-<4} {:-<4} {:-<7} {:-<5}",
            "", "", "", "", "", "", "", "",
        );      
        for result in results {
            println!("{:40} {:4} {:4} {:4} {:4} {:4} {:6.2}% {:5}",
                result.name.chars().take(40).collect::<String>(), 
                result.submitted, result.missing, result.excused,
                result.ungraded_init, result.ungraded_resubmit,
                result.curr_score, result.curr_grade
            );
        }

        Ok(())
    }

    async fn grades(&self, course : i32) -> Result<(), String> {
        #[derive(sqlx::FromRow)]
        struct Query {
            name : String,
            submitted : i64,
            missing : i64,
            excused : i64,
            grade_a : i64,
            grade_b : i64,
            grade_c : i64,
            grade_d : i64,
            grade_f : i64,
            grade_zero : i64,
            avg_score : Option<f64>,
            avg_grade : Option<f64>,
            avg_grade_nonzero : Option<f64>,
            group : i32,
            ungraded_init : i64,
            ungraded_resubmit : i64
        }
        let results = sqlx::query_as::<_,Query>(
                "
                SELECT 
                    asn.name,
                    SUM(CASE WHEN sub.attempt > 0 THEN 1 ELSE 0 END) as submitted,
                    SUM(CASE WHEN sub.missing THEN 1 ELSE 0 END) as missing,
                    SUM(CASE WHEN sub.excused THEN 1 ELSE 0 END) as excused,
                    SUM(CASE WHEN sub.current_submission THEN 0 ELSE 1 END) as something,
                    SUM(CASE WHEN sub.score >= (0.9 * asn.points_possible) THEN 1 ELSE 0 END) as grade_a,
                    SUM(CASE WHEN sub.score >= (0.8 * asn.points_possible) and 
                                  sub.score < (0.9 * asn.points_possible) THEN 1 ELSE 0 END) as grade_b,
                    SUM(CASE WHEN sub.score >= (0.7 * asn.points_possible) and 
                                  sub.score < (0.8 * asn.points_possible) THEN 1 ELSE 0 END) as grade_c,
                    SUM(CASE WHEN sub.score >= (0.6 * asn.points_possible) and 
                                  sub.score < (0.7 * asn.points_possible) THEN 1 ELSE 0 END) as grade_d,
                    SUM(CASE WHEN sub.score > 0 and 
                                  sub.score < (0.6 * asn.points_possible) THEN 1 ELSE 0 END) as grade_f,
                    SUM(CASE WHEN sub.score = 0 THEN 1 ELSE 0 END) as grade_zero,
                    AVG(sub.score) as avg_score,
                    CASE WHEN asn.points_possible = 0 THEN 0 
                         ELSE AVG(sub.score) / asn.points_possible * 100 END as avg_grade,
                    CASE WHEN asn.points_possible = 0 THEN 0 
                         ELSE AVG(CASE WHEN sub.score > 0 THEN sub.score ELSE NULL END) / asn.points_possible * 100 END as avg_grade_nonzero,
                    asn.assignment_group_id as group,
                    SUM(CASE WHEN sub.score IS NULL and sub.attempt > 0 THEN 1 ELSE 0 END) as ungraded_init,
                    SUM(CASE WHEN not sub.current_submission THEN 1 ELSE 0 END) as ungraded_resubmit
                FROM curr_assignments AS asn
                INNER JOIN curr_submissions AS sub
                    ON asn.id = sub.assignment_id
                INNER JOIN curr_students as stu
                    ON stu.id = sub.user_id
                WHERE asn.course_id = $1 and stu.course_id = $1 and asn.points_possible > 0
                GROUP BY asn.name, asn.points_possible, asn.assignment_group_id
                ORDER BY asn.assignment_group_id, asn.name;
            ")
            .bind(course)
            .fetch_all(&self.database)
            .await
            .map_err(|e| err!("Course List SQL Query Failure",e))?;
        println!("{:40} {:4} {:4} {:4} {:4} {:4} {:4} {:4} {:4} {:4} {:4} {:4} {:7} {:8} {:11}",
            "ASSIGNMENT","SUBM","MISS","EXCU", "UG-I", "UG-R", "GR-A", "GR-B", "GR-C", "GR-D", "GR-F", "ZERO", "AVG-SCR", "AVG-GRD%", "AVG-GRD-N0%"
        );
        let mut prev_group = -1;
        for result in results {
            if prev_group != result.group {
                println!("{:-<40} {:-<4} {:-<4} {:-<4} {:-<4} {:-<4} {:-<4} {:-<4} {:-<4} {:-<4} {:-<4} {:-<4} {:-<7} {:-<8} {:-<11}",
                    "", "", "", "", "", "", "", "", "", "", "", "", "", "",""
                );      
                prev_group = result.group;
            }
            println!("{:40} {:4} {:4} {:4} {:4} {:4} {:4} {:4} {:4} {:4} {:4} {:4} {:7.2} {:7.2}% {:10.2}%",
                result.name.chars().take(40).collect::<String>(), 
                result.submitted, result.missing, result.excused,
                result.ungraded_init, result.ungraded_resubmit,
                result.grade_a, result.grade_b, result.grade_c, result.grade_d,
                result.grade_f, result.grade_zero, 
                result.avg_score.unwrap_or(0.0),
                result.avg_grade.unwrap_or(0.0),
                result.avg_grade_nonzero.unwrap_or(0.0)
            );
        }

        Ok(())

    }

    


}