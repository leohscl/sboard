use super::job_parser::JobFields;
use super::job_parser::JobState;
use crate::app::App;
use crate::app::FetchTime;
use crate::job_query_info::JobQueryInfo;
use crate::job_query_info::JobTime;
use crate::jobs::job_parser::NumberOrCol;
use crate::parser::RunMode;
use clap::ValueEnum;
use color_eyre::eyre::Result;
use std::process::Command;
use tracing::info;

static FORMAT_STR: &str = "--format=JobID,JobName,Partition,Account,AllocCPUS,State,ExitCode,SubmitLine%50,WorkDir%100,Submit%20,ReqMem,MaxRSS";
// "JobIDRaw",
// "JobID",
// "State",
// "AllocCPUS",
// "TotalCPU",
// "Elapsed",
// "Timelimit",
// "REQMEM",
// "MaxRSS",
// "NNodes",
// "NTasks",
// "Partition",
// Account             AdminComment        AllocCPUS           AllocNodes
// AllocTRES           AssocID             AveCPU              AveCPUFreq
// AveDiskRead         AveDiskWrite        AvePages            AveRSS
// AveVMSize           BlockID             Cluster             Comment
// Constraints         ConsumedEnergy      ConsumedEnergyRaw   Container
// CPUTime             CPUTimeRAW          DBIndex             DerivedExitCode
// Elapsed             ElapsedRaw          Eligible            End
// ExitCode            Extra               FailedNode          Flags
// GID                 Group               JobID               JobIDRaw
// JobName             Layout              Licenses            MaxDiskRead
// MaxDiskReadNode     MaxDiskReadTask     MaxDiskWrite        MaxDiskWriteNode
// MaxDiskWriteTask    MaxPages            MaxPagesNode        MaxPagesTask
// MaxRSS              MaxRSSNode          MaxRSSTask          MaxVMSize
// MaxVMSizeNode       MaxVMSizeTask       McsLabel            MinCPU
// MinCPUNode          MinCPUTask          NCPUS               NNodes
// NodeList            NTasks              Partition           Planned
// PlannedCPU          PlannedCPURAW       Priority            QOS
// QOSRAW              Reason              ReqCPUFreq          ReqCPUFreqGov
// ReqCPUFreqMax       ReqCPUFreqMin       ReqCPUS             ReqMem
// ReqNodes            ReqTRES             Reservation         ReservationId
// Start               State               Submit              SubmitLine
// Suspended           SystemComment       SystemCPU           Timelimit
// TimelimitRaw        TotalCPU            TRESUsageInAve      TRESUsageInMax
// TRESUsageInMaxNode  TRESUsageInMaxTask  TRESUsageInMin      TRESUsageInMinNode
// TRESUsageInMinTask  TRESUsageInTot      TRESUsageOutAve     TRESUsageOutMax
// TRESUsageOutMaxNode TRESUsageOutMaxTask TRESUsageOutMin     TRESUsageOutMinNode
// TRESUsageOutMinTask TRESUsageOutTot     UID                 User
// UserCPU             WCKey               WCKeyID             WorkDir

fn run_sacct(run_mode: RunMode, hours_before_now: u16) -> Result<String> {
    let fmt_time = format!("now-{}hours", hours_before_now);
    let sacct_args = vec![FORMAT_STR, "-P", "-S", &fmt_time];
    run_command(run_mode, "sacct", &sacct_args)
}

fn update_max_rss(job_fields: &mut JobFields, all_job_fields: &[JobFields]) {
    job_fields.maxrss = NumberOrCol::Value(
        all_job_fields
            .iter()
            .filter_map(|f| {
                info!("does it start with ? {:?}", f);
                if f.job_id.starts_with(&job_fields.job_id) {
                    info!("yes");
                    Some(f.maxrss.clone().take().unwrap())
                } else {
                    info!("no");
                    None
                }
            })
            .max()
            .expect("Sums max rss"),
    );
}

pub fn fetch_jobs(app: &App, job_info: JobQueryInfo) -> Result<Vec<JobFields>> {
    let cli = &app.cli;
    let hours_before_now = match app.fetch_time {
        FetchTime::Today => 24,
        FetchTime::ThreeDaysAgo => 24 * 3,
        FetchTime::AWeekAgo => 24 * 7,
        FetchTime::SpecificWindow { .. } => todo!(),
    };
    let sacct_res = run_sacct(cli.run_mode, hours_before_now)?;
    let all_job_fields = JobFields::from_sacct_str(&sacct_res)?;
    info!("{:?}", all_job_fields);
    // remove fields with empty partition
    let mut job_fields_with_partition = all_job_fields.clone();
    job_fields_with_partition.retain(|job_fields| !job_fields.partition.is_empty());
    // get maxRSS info from job steps
    job_fields_with_partition
        .iter_mut()
        .filter(|j| j.job_id != "JobID")
        .for_each(|job_fields| update_max_rss(job_fields, &all_job_fields));
    match job_info.time {
        JobTime::Running => {
            job_fields_with_partition.retain(|job_fields| {
                matches!(job_fields.state, JobState::Running | JobState::Header)
            });
        }
        JobTime::Finished => job_fields_with_partition.retain(|job_fields| {
            !matches!(job_fields.state, JobState::Running)
                | matches!(job_fields.state, JobState::Header)
        }),
        JobTime::All => (),
    }
    job_fields_with_partition[1..].sort_by(|f1, f2| f1.submit.cmp(&f2.submit).reverse());
    let capped = false;
    let job_fields = if capped {
        job_fields_with_partition
            .into_iter()
            .take(app.cli.job_max_display as usize)
            .collect()
    } else {
        job_fields_with_partition
    };
    Ok(job_fields)
}

pub fn get_log_files_finished_job(
    run_mode: RunMode,
    workdir: &str,
    job_id: &str,
) -> Result<String> {
    let regex_id = if job_id.contains('[') {
        job_id.split('[').next().unwrap().to_string() + "_*"
    } else {
        job_id.to_string()
    };
    let regex = String::from("*") + &regex_id + "*";
    // info!("regex: {}", regex);
    let find_args = [workdir, "-maxdepth", "2", "-name", &regex];
    run_command(run_mode, "find", &find_args)
}

pub fn read_file(run_mode: RunMode, path: &str) -> Result<String> {
    let cat_args = vec![path];
    run_command(run_mode, "cat", &cat_args)
}

fn run_command(run_mode: RunMode, cmd: &str, command_args: &[&str]) -> Result<String> {
    let output_jobs = match run_mode {
        RunMode::Slurm => Command::new(cmd).args(command_args).output()?,
        RunMode::FromFile => Command::new("/bin/cat")
            .arg("test_data/10_random_users.txt")
            .output()?,
        RunMode::Ssh => Command::new("ssh")
            .arg("maestro")
            .arg(cmd)
            .args(command_args)
            .output()?,
    };
    let out_txt = String::from_utf8(output_jobs.stdout)?;
    Ok(out_txt)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum DisplayMode {
    Cpu,
    Ram,
}
