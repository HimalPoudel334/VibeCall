// Define templates once using lazy_static
lazy_static::lazy_static! {
    pub static ref TEMPLATES: tera::Tera = {
        let mut tera = match tera::Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Template parsing error(s): {}", e);
                std::process::exit(1);
            }
        };
        // Enable autoescaping for HTML files
        tera.autoescape_on(vec![".html", ".htm"]);
        tera
    };
}
