use std::collections::HashMap;
use std::env::consts::OS;
use std::process::{exit, Stdio};

use surrealdb::engine::remote::http::Http;
use surrealdb::opt::auth::Root;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{error, warn};

use crate::utils::config::get_global_config;
use crate::utils::error::Error;
use crate::world::chunkformat::Chunk;

mod chunkformat;
pub mod importing;

pub async fn start_database() -> Result<(), Error> {
    let store_path = if get_global_config().database.mode == "file" {
        format!("file:{}", get_global_config().database.path)
    } else {
        "memory".to_string()
    };
    let envs: HashMap<&str, String> = HashMap::from([
        (
            "SURREAL_BIND",
            format!(
                "127.0.0.1:{}",
                get_global_config().database.port.to_string()
            ),
        ),
        ("SURREAL_PATH", store_path),
        ("SURREAL_LOG_LEVEL", "info".to_string()),
        ("SURREAL_NO_BANNER", "true".to_string()),
    ]);

    let mut executable_name = match OS {
        "windows" => ".\\surreal.exe",
        "macos" => "./surreal",
        "linux" => "./surreal",
        _ => {
            return Err(Error::Generic("Unsupported OS".to_string()));
        }
    };

    if !tokio::fs::try_exists(executable_name).await.unwrap() {
        if which::which("surreal").is_ok() {
            warn!("Using system surreal executable. This may be outdated.");
            executable_name = "surreal";
        } else {
            error!("Surreal executable not found at {}", executable_name);
            exit(1);
        }
    }

    // start memory -A --auth --user root --pass root
    let mut surreal_setup_process = tokio::process::Command::new(executable_name)
        .arg("start")
        // .args(&["-A", "--auth", "--user", "ferrumc", "--pass", "ferrumc"])
        .args(&["-A", "--auth"])
        .envs(envs)
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let db =
        surrealdb::Surreal::new::<Http>(format!("127.0.0.1:{}", get_global_config().database.port))
            .await
            .unwrap();
    db.signin(Root {
        username: "ferrumc",
        password: "ferrumc",
    })
    .await
    .unwrap();
    db.use_ns("ferrumc").await.unwrap();
    db.use_db(get_global_config().world.clone()).await.unwrap();

    let query = "\
        DEFINE TABLE chunks;
        DEFINE TABLE entities;";
    db.query(query).await.unwrap();

    // Why the fuck does surrealdb use stderr by default???
    let mut stderr = BufReader::new(surreal_setup_process.stderr.take().unwrap());
    let mut output = String::new();
    loop {
        stderr.read_line(&mut output).await.unwrap();
        if !output.is_empty() {
            print!("{output}");
            output.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use fastnbt::Value;

    use crate::world;
    use crate::world::load_chunk;

    #[tokio::test]
    async fn dump_region_to_json() {
        let f = std::fs::File::open("./dummyregion.mca").unwrap();
        let mut reader = fastanvil::Region::from_stream(f).unwrap();
        let chunk = reader.read_chunk(0, 0).unwrap().unwrap();
        let chunk_nbt: Value = fastnbt::from_bytes(&chunk).unwrap();
        let mut outfile = std::fs::File::create("chunk.json").unwrap();
        let raw_nbt = serde_json::ser::to_vec(&chunk_nbt).unwrap();
        outfile.write_all(&*raw_nbt).unwrap()
    }

    #[tokio::test]
    async fn chunk_to_struct() {
        let chunk = load_chunk(0, 0).await.unwrap();
        assert_eq!(chunk.x_pos, 0);
        assert_eq!(chunk.z_pos, 0);
        assert_eq!(chunk.y_pos, -4);
    }
}

pub async fn load_chunk(x: i32, z: i32) -> Result<Chunk, Error> {
    // TODO: Replace with database call when that is all set up
    let region_area = (
        (x as f64 / 32.0).floor() as i32,
        (z as f64 / 32.0).floor() as i32,
    );
    let region_file = std::fs::File::open("dummyregion.mca")?;
    let mut region = fastanvil::Region::from_stream(region_file).unwrap();
    let raw_chunk_data = region
        .read_chunk(x as usize, z as usize)
        .map_err(|_| {
            Error::Generic(format!(
                "Unable to read chunk {} {} from region {} {} ",
                x, z, region_area.0, region_area.1
            ))
        })?
        .expect(
            format!(
                "Chunk {} {} not found in region {} {}",
                x, z, region_area.0, region_area.1
            )
            .as_str(),
        );
    fastnbt::from_bytes(&raw_chunk_data).map_err(|_| {
        Error::Generic(format!(
            "Unable to parse chunk {} {} from region {} {} ",
            x, z, region_area.0, region_area.1
        ))
    })
}
