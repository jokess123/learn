use bytesize::ByteSize;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::vec;
use sysinfo::Disks;
use toml;

fn main() {
    println!("Hello, world!");

    disk_info();

    let config = Config {
        disk_farms: vec![
            DiskFarmInner {
                directory: PathBuf::from("/tmp/plot"),
                allocated_plotting_space: Some("1500G".to_string()),
            },
            DiskFarmInner {
                directory: PathBuf::from("/tmp/plot1"),
                allocated_plotting_space: None,
            },
        ],
        plot_servers: vec!["localhost:12345".to_string()],
    };

    let contents = toml::to_string(&config).expect("Failed to serialize config");
    fs::write(
        "/Users/haoziyan/Desktop/code/github.com/jokess123/learn/hello/config.toml",
        contents,
    )
    .expect("Failed to write config");

    let cfg =
        load_config("/Users/haoziyan/Desktop/code/github.com/jokess123/learn/hello/config.toml")
            .expect("Failed to load config");
    println!("{:#?}", cfg);

    let disk_farms = cfg.get_disk_farms();
    disk_farms.into_iter().for_each(|disk_farm| {
        println!(
            "{:?} {:?}",
            disk_farm.directory, disk_farm.allocated_plotting_space
        );
    });
}

fn disk_info() {
    // We display all disks' information:
    println!("=> disks:");
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        println!("{disk:?}");
        println!("{:?}", disk.name());
        println!("{:?}", disk.kind());
        println!("{:?}", disk.file_system());
        println!("{:?}", disk.mount_point());
        println!("{:?}", disk.total_space());
        println!("{:?}", disk.available_space());
        println!("{:?}", disk.is_removable());
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DiskFarm {
    /// Path to directory where data is stored.
    directory: PathBuf,
    /// How much space in bytes can farm use for plots (metadata space is not included)
    allocated_plotting_space: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiskFarmInner {
    /// Path to directory where data is stored.
    directory: PathBuf,
    /// How much space in bytes can farm use for plots (metadata space is not included)
    allocated_plotting_space: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    disk_farms: Vec<DiskFarmInner>,
    plot_servers: Vec<String>,
}

fn load_config(config_file: &str) -> Result<Config, Box<dyn Error>> {
    let contents = fs::read_to_string(config_file)?;
    let config: Config = toml::from_str(&contents)?;

    if has_dup(&config.plot_servers) {
        return Err("Plot servers must be unique".into());
    }

    let dirs: Vec<PathBuf> = config
        .disk_farms
        .iter()
        .map(|disk_farm| disk_farm.directory.clone())
        .collect();
    if has_dup(&dirs) {
        return Err("Disk farms must be unique".into());
    }

    Ok(config)
}

fn has_dup<T: PartialEq>(slice: &[T]) -> bool {
    for i in 1..slice.len() {
        if slice[i..].contains(&slice[i - 1]) {
            return true;
        }
    }
    false
}

impl Config {
    fn get_disk_farms(&self) -> Vec<DiskFarm> {
        self.disk_farms
            .iter()
            .map(|disk_farm| DiskFarm {
                directory: disk_farm.directory.clone(),
                allocated_plotting_space: if disk_farm.allocated_plotting_space.is_none() {
                    None
                } else {
                    Some(
                        ByteSize::from_str(&disk_farm.allocated_plotting_space.clone().unwrap())
                            .expect("Failed to parse disk_farm.allocated_plotting_space")
                            .as_u64(),
                    )
                },
            })
            .collect()
    }
}
