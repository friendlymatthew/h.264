use std::process::Command;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    file_path: String,
}

fn pull_h264(input_file: &str, output_file: &str) -> Result<()> {
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_file)
        .arg("-c:v")
        .arg("libx264")
        .arg("-crf")
        .arg("23")
        .arg(output_file)
        .status()?;

    if !status.success() {
        anyhow::bail!("ffmpeg failed with exit code: {}", status);
    }

    Ok(())
}

fn clean_file_path(path: &str) -> String {
    let file_name = path.rsplit('/').next().unwrap_or(path);
    match file_name.rsplit('.').collect::<Vec<&str>>().as_slice() {
        ["zip", this] | ["mp4", this] => {
            println!("this: {}", this);
            this.to_string()
        }
        _ => file_name.to_string(),
    }
}

fn main() -> Result<()> {
    let Args { file_path } = Args::parse();
    let f_title = clean_file_path(&file_path);
    let output_file = format!("{}.h264", f_title);
    pull_h264(&file_path, &output_file)?;
    println!("Conversion complete: {}", output_file);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string() -> Result<()> {
        let fs = vec![
            ("thedog/licked/theole/and/everyone/laughed", "laughed"),
            ("a/b/c/d.zip", "d"),
            ("a/.zip", ""),
            ("a/wef.mp4", "wef"),
            ("wef.mp4", "wef"),
        ];

        for (fp, et) in fs {
            let ft = clean_file_path(fp);
            assert_eq!(ft, et);
        }

        Ok(())
    }
}
