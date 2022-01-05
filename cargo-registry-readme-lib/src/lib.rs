use cargo_registry_markdown::text_to_html;

// here we want to convert the readme string to html, write it to disk and then open the browser
// with the chosen wireframe template
pub fn preview(readme: &str) -> String {
    text_to_html(readme, "README.md", None, None)
}

#[cfg(test)]
mod tests {
    use super::preview;
    use std::fs;

    #[test]
    fn it_works() -> anyhow::Result<()> {
        //      let readme_str =
        //          fs::read_to_string("/home/rodzilla/Documents/Projects/rusty_paseto/readme.md")?;
        //      preview(&readme_str)?;

        Ok(())
    }
}
