use reqwest;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubReleaseAsset>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubReleaseAsset {
    name: String,
    url: String,
}

use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::os::unix::prelude::PermissionsExt;

fn download_package(url: &str, filename: &str) -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10_5_9) AppleWebKit/600.31 (KHTML, like Gecko) Chrome/54.0.2042.261 Safari/536")
        .build()?;

    let response = client.get(url).send()?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let content = response.bytes()?; // Use bytes() instead of text() for binary data
            let mut file = File::create(filename)?;

            // Write the content to the file
            file.write_all(&content)?;

            // Set executable permissions on the file (Unix-like systems only)
            #[cfg(unix)]
            {
                let mut permissions = file.metadata()?.permissions();
                permissions.set_mode(0o755); // This sets the permission to rwxr-xr-x
                std::fs::set_permissions(filename, permissions)?;
            }

            Ok(())
        }
        status => {
            eprintln!("Error: {}", status);
            Err("Failed to fetch or save package".into())
        }
    }
}

fn get_latest_release(username: &str, reponame: &str) -> Result<GitHubRelease, Box<dyn Error>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        username, reponame
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10_5_9) AppleWebKit/600.31 (KHTML, like Gecko) Chrome/54.0.2042.261 Safari/536")
        .build()?;

    let response = client.get(&url).send()?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let release: GitHubRelease = response.json()?;
            Ok(release)
        }
        status => {
            eprintln!("Error: {}", status);
            Err("Failed to fetch release information".into())
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let args: Vec<_> = args[1..].to_vec();

    if args.len() != 1 {
        println!("Usage: ./spm <username>/<reponame>");
        return;
    }

    let repo = args[0].clone();
    let repo: Vec<_> = repo.split("/").collect();
    let username = repo[0];
    let reponame = repo[1];

    match get_latest_release(username, reponame) {
        Ok(release) => {
            println!("Latest release tag: {}", release.tag_name);
            for asset in &release.assets {
                if asset.name.ends_with(".shed") {
                    let url = format!(
                        "https://github.com/{}/{}/releases/download/{}/{}",
                        username, reponame, release.tag_name, asset.name
                    );
                    println!("Downloading package from url: {}", url);
                    match download_package(&url, &asset.name) {
                        Ok(_) => {
                            println!("Downloaded package: {}", asset.name);
                        }
                        Err(err) => {
                            eprintln!("Error: {:?}", err);
                        }
                    }

                    return;
                }
            }
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
        }
    }
}
