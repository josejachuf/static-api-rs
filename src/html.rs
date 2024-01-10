use salvo::prelude::*;
use std::path::PathBuf;
use crate::AppConfig;

#[handler]
pub async fn index(res: &mut Response, depot: &mut Depot) -> Result<(), anyhow::Error> {
    let app_config = depot.obtain::<AppConfig>().unwrap().clone();
    let data_dir = &app_config.data_dir;

    let mut data_content = tokio::fs::read_dir(data_dir).await?;
    let mut data_files: Vec<PathBuf> = Vec::new();

    while let Some(data_input) = data_content.next_entry().await? {
        let ruta = data_input.path();
        let name = data_input.file_name();
        if ruta.is_file() {
            data_files.push(name.into());
        }
    }

    let data_files: Vec<String> = data_files
        .into_iter()
        .filter_map(|path| {
            path.file_stem().map(|stem| {
                format!(
                    r#"<li><a href="/api/{}">{}</a></li>"#,
                    stem.to_string_lossy().to_string(),
                    stem.to_string_lossy().to_string()
                )
            })
        })
        .collect();

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>Static API</title>
            </head>
            <body>
                <h1>Static API</h1>

                <p>This is a simple application simulating a basic REST API. It allows CRUD operations (Create, Read, Update, Delete) on different collections, where each collection is represented as a JSON file in the file system. If the collection does not exist, it is automatically created.</p>

                <h3>Collections</h3>

                <ul>{}</ul>

                <p>JSON files are stored in <strong>{data_dir}<strong></p>

                <h3>Try something like</h3>

                <div style="background-color: #DEDEDE; padding: 10px;">

                    <pre>curl -X GET http://localhost:5800/api/&lt;collection&gt;</pre>

                    <pre>curl -X GET http://localhost:5800/api/&lt;collection&gt;/&lt;id&gt;</pre>

                    <pre>curl -X POST -H "Content-Type: application/json" -d '{{"field1":"value1", "field2":"value2"}}' http://localhost:5800/api/&lt;collection&gt;</pre>

                    <pre>curl -X PUT -H "Content-Type: application/json" -d '{{"field1":"new_value1", "new_field2":"value2"}}' http://localhost:5800/api/&lt;collection&gt;/&lt;id&gt;</pre>

                    <pre>curl -X DELETE http://localhost:5800/api/&lt;collection&gt;/&lt;id&gt;</pre>
                </div>

            </body>
        </html>
        "#,
        data_files.join("")
    );

    res.render(Text::Html(html));
    Ok(())
}
