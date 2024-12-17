use std::{fs, io};

pub struct RenderFlags{
    pub fs_path: String,
}

impl Default for RenderFlags{
    fn default() -> Self {
        RenderFlags{
            fs_path: String::from("/")
        }
    }
}

/// Will search for an identifier and replace it with specific data.
/// Can be thought of as a rudimentary Jinja implementation.
/// 
/// Replacing is done in passes and in-place.
/// 
/// Supported identifiers:
/// - {{files_list}}
pub fn render_index_page(page: &mut String, flags: &RenderFlags) -> Result<(), std::io::Error>{
    
    // Supported identifiers must be added here and handled below
    let identifiers = ["{{files_list}}", "{{up_level_link}}"];
    let mut tokens;

    // This is quite an inefficient way of doing this.
    // However, this project does not aim to render pages with multiple
    // identifiers, neither with multiple versions of such identifier
    for id in identifiers{
        tokens = page.split(id).collect::<Vec<&str>>();

        // Identifier not found
        if tokens.len() < 2 {continue;}

        // Render and stitch back together
        let mut list_html;
        match id {
            "{{files_list}}" => {
                list_html = String::from("<ul>");

                let entries = fs::read_dir(&flags.fs_path)?;
                for entry in entries{
                    let entry = entry?;

                    list_html += "<li>";
                    
                    // File name
                    if let Some(name) = entry.path().file_name(){
                        let file_name = name.to_str().unwrap_or_default();
                        if entry.file_type()?.is_dir(){
                            
                            // Determine the correct url to reference
                            let mut href_path = String::from("/fs");
                            if flags.fs_path != "/"{
                                href_path.push_str(flags.fs_path.as_str());
                            }
                            href_path = format!("{href_path}/{file_name}");
                            
                            list_html += format!("<a href={href_path}>{file_name}</a>").as_str();
                        }
                        else{
                            list_html += file_name;
                        }
                    }

                    list_html += "</li>";
                }

                list_html += "</ul>";
            },
            "{{up_level_link}}" => {
                list_html = String::from("<a href=/fs");

                let fs_path = flags.fs_path
                    .split("/")
                    .collect::<Vec<&str>>();

                list_html += &fs_path[..fs_path.len()-1].join("/");
                
                list_html += ">../</a>";
            }
            _ => {
                return Err(io::Error::new(io::ErrorKind::NotFound, "Error rendering file"))
            }
        }

        *page = tokens.join(&list_html);
    }

    Ok(())
}