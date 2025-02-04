use regex::Regex;
use tokio::fs;

pub struct RenderFlags {
    pub fs_path: String,
}

impl Default for RenderFlags {
    fn default() -> Self {
        RenderFlags {
            fs_path: String::from("/"),
        }
    }
}

#[derive(Debug)]
pub enum RenderError {
    InvalidId(String),
    PathOutsideBaseFsPath,
    FilesList,
}

/// Will search for an identifier and replace it with specific data.
/// Can be thought of as a rudimentary Jinja implementation.
///
/// Replacing is done in passes and in-place.
///
/// Supported identifiers:
/// - {{files_list}}
/// - {{up_level_link}}
/// - {{curr_path}}
pub async fn render_index_page(
    page: &mut String,
    flags: &RenderFlags,
    base_fs_path: &str,
) -> Result<(), RenderError> {
    // Supported identifiers must be added here and handled below
    let identifiers = ["files_list", "up_level_link", "curr_path"];
    let mut tokens;

    // This is quite an inefficient way of doing this.
    // However, this project does not aim to render pages with multiple
    // identifiers, neither with multiple versions of such identifier
    for id in identifiers {
        let rule = "\\{\\{".to_owned() + id + "\\}\\}";
        let re = Regex::new(&rule).unwrap();

        tokens = re.split(page).collect::<Vec<&str>>();

        // Identifier not found
        if tokens.len() < 2 {
            continue;
        }

        // Render and stitch back together
        let list_html = match id {
            "files_list" => {
                render_files_list(&flags.fs_path, base_fs_path).await.unwrap_or_else(|e| 
                    match e {
                        RenderError::FilesList => "<br>Unable to read into directory".to_string(),
                        _ => "".to_string(),
                    }
                )
            }
            "up_level_link" => {
                render_up_level_link(&flags.fs_path, base_fs_path).unwrap_or_else(|e| 
                    match e {
                        RenderError::PathOutsideBaseFsPath => format!(
                            "Access denied. Go back to <a href=/fs{base_fs_path}>{base_fs_path}</a>"
                        ),
                        _ => "".to_string(),
                    }
                )
            }
            "curr_path" => flags.fs_path.to_string(),
            _ => {
                log::error!("INVALID ID");
                return Err(RenderError::InvalidId(format!(
                    "Error rendering file. Unsupported identifier: {id}"
                )));
            }
        };

        *page = tokens.join(&list_html);
    }

    Ok(())
}

async fn render_files_list(path: &str, base_fs_path: &str) -> Result<String, RenderError> {
    // Invalid path
    if !path.starts_with(base_fs_path) {
        return Err(RenderError::PathOutsideBaseFsPath);
    }

    let mut entries = fs::read_dir(path)
        .await
        .map_err(|_| RenderError::FilesList)?;

    let mut output = String::from("<ul>");

    while let Some(entry) = entries.next_entry().await.map_err(|_| RenderError::FilesList)?{
        output += "<li>";

        // File name
        if let Some(name) = entry.path().file_name() {
            let file_name = name.to_str().unwrap_or_default();

            let entry_file_type = entry.file_type().await;
            if entry_file_type.is_err() {
                continue;
            }
            if entry_file_type.unwrap().is_dir() {
                // Determine the correct url to reference
                let mut href_path = String::from("/fs");
                if path != "/" {
                    href_path += path;
                }
                href_path = format!("{href_path}/{file_name}");

                output += format!("<a href={href_path}>{file_name}</a>").as_str();
            } else {
                output += file_name;
            }
        }

        output += "</li>";
    }

    output += "</ul>";

    Ok(output)
}

fn render_up_level_link(path: &str, base_fs_path: &str) -> Result<String, RenderError> {
    // Invalid path
    if !path.starts_with(base_fs_path) {
        return Err(RenderError::PathOutsideBaseFsPath);
    }

    let mut output = String::from("<a href=/fs");

    let fs_path = path.split("/").collect::<Vec<&str>>();

    output += &fs_path[..fs_path.len() - 1].join("/");

    output += ">../</a>";
    Ok(output)
}
