use crate::jobs::job_parser::JobFields;
use crate::App;

#[derive(Clone, Debug)]
pub struct JobQueryInfo {
    pub refresh: bool,
    pub job_list: Vec<JobFields>,
    pub time: JobTime,
    pub changed: bool,
    pub job_display: Vec<JobFields>,
    pub folded_jobs: Vec<bool>,
}

#[derive(Clone, Debug)]
pub enum JobTime {
    Finished,
    Running,
    All,
}

impl JobQueryInfo {
    pub fn from_result(job_list: Vec<JobFields>, app: &App) -> Self {
        let mut jqi = JobQueryInfo {
            refresh: app.cli.refresh,
            time: JobTime::All,
            job_list: job_list.clone(),
            changed: false,
            folded_jobs: vec![false; job_list.len()],
            job_display: vec![],
        };
        jqi.make_display();
        // info!("{:?}", &jqi.job_display);
        jqi
    }

    pub fn make_display(&mut self) {
        let mut job_display = vec![];
        let mut opt_job_array_display: Option<JobArrayDisplay> = None;
        self.job_list.iter().for_each(|j| {
            if j.job_id.contains('_') && !j.job_id.contains('[') {
                let mut split = j.job_id.split('_');
                let array_jid = split.next().unwrap();
                let array_num = split.next().unwrap().parse::<u32>().unwrap();
                opt_job_array_display = match opt_job_array_display.clone() {
                    None => Some(JobArrayDisplay::new(array_jid, array_num, j.clone())),
                    Some(mut jobarr) => {
                        jobarr.update(array_num);
                        Some(jobarr)
                    }
                }
            } else {
                if let Some(jobarr) = opt_job_array_display.clone() {
                    job_display.push(jobarr.get_as_field());
                    opt_job_array_display = None;
                }
                job_display.push(j.clone())
            }
        });
        if let Some(jobarr) = opt_job_array_display {
            job_display.push(jobarr.get_as_field());
        }
        self.job_display = job_display;
    }

    pub fn default(app: &App) -> Self {
        JobQueryInfo {
            refresh: app.cli.refresh,
            job_list: Vec::new(),
            time: JobTime::All,
            changed: false,
            job_display: Vec::new(),
            folded_jobs: Vec::new(),
        }
    }
}
#[derive(Clone)]
struct JobArrayDisplay {
    min: u32,
    max: u32,
    id: String,
    job_field: JobFields,
}

impl JobArrayDisplay {
    fn new(id: &str, num: u32, job_field: JobFields) -> Self {
        JobArrayDisplay {
            min: num,
            max: num,
            id: id.to_string(),
            job_field,
        }
    }

    fn update(&mut self, num: u32) {
        if num > self.max {
            self.max = num;
        }
        if num < self.min {
            self.min = num
        }
    }

    fn get_as_field(mut self) -> JobFields {
        let name = self.id + "[" + &self.min.to_string() + "-" + &self.max.to_string() + "]";
        self.job_field.job_id = name;
        self.job_field
    }
}
