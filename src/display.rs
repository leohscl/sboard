use clap::ValueEnum;
use std::collections::HashMap;
use std::error::Error;

pub fn build_display(
    job_raw_output: String,
    display_mode: &DisplayMode,
) -> Result<Vec<String>, Box<dyn Error>> {
    let line_vector: Vec<Vec<String>> = job_raw_output
        .trim_end_matches("\n")
        .split("\n")
        .enumerate()
        .map(|(index, line)| {
            line.split("|")
                .map(|str| {
                    if index == 0 {
                        rename_field(str)
                    } else {
                        str.to_string()
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let mut fields_keep = display_mode.get_fields();
    let all_fields = &line_vector[0];
    let indicies: Vec<_> = all_fields
        .into_iter()
        .enumerate()
        .filter_map(|(index, field)| {
            if fields_keep.contains(&field) {
                fields_keep.retain(|f| f != field);
                Some(index)
            } else {
                None
            }
        })
        .collect();
    let job_lines = line_vector
        .into_iter()
        .map(|line_fields| {
            let line = line_fields
                .into_iter()
                .enumerate()
                .filter_map(|(index, field)| {
                    if indicies.contains(&index) {
                        let field_fmt = format!("{:12}", field);
                        Some(field_fmt)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            line
        })
        .collect();
    Ok(job_lines)
}

fn rename_field(field_raw: &str) -> String {
    let mut rename_dict = HashMap::new();
    rename_dict.insert("NODELIST(REASON)", "NODE".to_string());
    match rename_dict.get(field_raw) {
        Some(new_field) => new_field.clone(),
        None => field_raw.to_string(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum DisplayMode {
    CPU,
    RAM,
}

impl DisplayMode {
    fn get_fields(&self) -> Vec<String> {
        let default_fields = [
            "JOBID",
            "PARTITION",
            "NAME",
            "USER",
            "STATE",
            "TIME",
            "TIME_LIMI",
            "NODE",
        ];
        let mut specific_fields = match self {
            DisplayMode::CPU => vec!["NODES"],
            DisplayMode::RAM => vec![""],
        };
        let mut fields = default_fields.to_vec();
        fields.append(&mut specific_fields);
        fields.into_iter().map(|str| str.to_string()).collect()
    }
}
