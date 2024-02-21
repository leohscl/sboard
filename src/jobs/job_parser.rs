use color_eyre::Result;
use phf::phf_map;
use tracing::info;

use crate::{jobs::job_handler, parser::RunMode};

static SACCT_MAP: phf::Map<&'static str, usize> = phf_map! {
    "JobID" => 0,
    "JobName" => 1,
    "Partition" => 2,
    "Account" => 3,
    "AllocCPUS" => 4,
    "State" => 5,
    "ExitCode" => 6,
    "SubmitLine" => 7,
    "WorkDir" => 8,
};

#[derive(Clone)]
pub struct JobFields {
    pub job_id: String,
    pub job_name: String,
    pub partition: String,
    pub account: String,
    pub alloc_cpus: String,
    pub state: String,
    pub exit_code: String,
    pub submit_line: String,
    pub workdir: String,
}

impl JobFields {
    pub fn from_slice(slice: Vec<String>) -> JobFields {
        assert_eq!(slice.len(), 9);
        JobFields {
            job_id: slice[0].clone(),
            job_name: slice[1].clone(),
            partition: slice[2].clone(),
            account: slice[3].clone(),
            alloc_cpus: slice[4].clone(),
            state: slice[5].clone(),
            exit_code: slice[6].clone(),
            submit_line: slice[7].clone(),
            workdir: slice[8].clone(),
        }
    }

    pub fn from_sacct_str(sacct_res: &str) -> Result<Vec<JobFields>> {
        //TODO(lhenches): use nom for parsing ?
        let line_vector: Vec<Vec<String>> = sacct_res
            .trim_end_matches('\n')
            .split('\n')
            .map(|line| {
                let raw_fields = line.split('|').map(|f| f.to_string()).collect::<Vec<_>>();
                assert_eq!(raw_fields.len(), SACCT_MAP.len());
                raw_fields
            })
            .collect();

        let all_fields = &line_vector[0];
        assert_eq!(all_fields.len(), SACCT_MAP.len());
        let hash_index: Vec<_> = all_fields
            .iter()
            .map(|field| {
                let index_final = SACCT_MAP[field];
                index_final
            })
            .collect();

        let fields_correct_order = line_vector
            .into_iter()
            .map(|line| {
                hash_index
                    .iter()
                    .map(|&i| line[i].clone())
                    .collect::<Vec<String>>()
            })
            .map(|line_correct_order| JobFields::from_slice(line_correct_order))
            .collect();

        Ok(fields_correct_order)
    }

    fn format_str(s: &str) -> String {
        format!("{:<30}", s)
    }

    pub fn display_lines(&self) -> String {
        [
            Self::format_str(&self.job_id),
            Self::format_str(&self.job_name),
            Self::format_str(&self.partition),
            Self::format_str(&self.account),
            Self::format_str(&self.alloc_cpus),
            Self::format_str(&self.state),
            Self::format_str(&self.exit_code),
            Self::format_str(&self.submit_line),
            Self::format_str(&self.workdir),
        ]
        .join(", ")
    }
}
pub fn fetch_logs(run_mode: RunMode, fields: &JobFields) -> Result<Vec<String>> {
    // try to get log file
    let find_result =
        job_handler::get_log_files_finished_job(run_mode, &fields.workdir, &fields.job_id)?;
    info!(find_result);
    // parse logs into multiple files
    let vec_logs = find_result
        .trim_end_matches('\n')
        .split('\n')
        .map(|s| s.to_string())
        .collect();

    Ok(vec_logs)
}
