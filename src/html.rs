use crate::AppConfig;
use salvo::prelude::*;
use std::path::PathBuf;
use crate::utils::delete_collection_sync;

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
                    r#"
<li class="columns is-gapless">
  <div class="column is-6">
    <a href="/api/{}" class="has-text-link is-size-5">{}</a>
  </div>
  <div class="column is-6">
    <a href="/delete-collection/{}" class="button is-danger is-small">delete</a>
  </div>
</li>

                    "#,
                    stem.to_string_lossy().to_string(),
                    stem.to_string_lossy().to_string(),
                    stem.to_string_lossy().to_string(),
                )
            })
        })
        .collect();

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css">
                <title>Static API</title>
            </head>
            <body>
                <section class="section">
                    <div class="container is-max-desktop">
                        <section class="hero is-info mb-5">
                          <div class="hero-body">
                            <p class="title">
                              Static-API
                            </p>
                            <p class="subtitle">
                              fake api for testing
                            </p>
                          </div>
                        </section>

                        <p class="has-text-justified">This is a simple application emulating a basic REST API. It allows CRUD operations (Create, Read, Update, Delete) on different collections, where each collection is represented as a JSON file in the file system. If the collection does not exist, it is automatically created.</p>

                        <h3 class="title is-4 has-text-grey-dark mt-4">Collections</h3>

                        <div class="content">
                            <ul>{}</ul>
                        </div>

                        <p>JSON files are stored in <strong>{data_dir}<strong></p>

                        <h3 class="title is-4 has-text-grey-dark mt-4">Try something like</h3>

                        <div class="box">
                          <code class="is-family-code p-2">
                            <p class="mb-4"><strong>curl -X GET</strong> http://localhost:5800/api/<span class="has-text-link">&lt;collection&gt;</span></p>
                            <p class="mb-4"><strong>curl -X GET</strong> http://localhost:5800/api/<span class="has-text-link">&lt;collection&gt;</span>?skip=10&limit=5</p>
                            <p class="mb-4"><strong>curl -X GET</strong> http://localhost:5800/api/<span class="has-text-link">&lt;collection&gt;</span>/<span class="has-text-link">&lt;id&gt;</span></p>
                            <p class="mb-4"><strong>curl -X POST</strong> -H "Content-Type: application/json" -d '{{"field1":"value1", "field2":"value2"}}' http://localhost:5800/api/<span class="has-text-link">&lt;collection&gt;</span></p>
                            <p class="mb-4"><strong>curl -X PUT</strong> -H "Content-Type: application/json" -d '{{"field1":"new_value1", "new_field2":"value2"}}' http://localhost:5800/api/<span class="has-text-link">&lt;collection&gt;</span>/<span class="has-text-link">&lt;id&gt;</span></p>
                            <p class="mb-4"><strong>curl -X DELETE</strong> http://localhost:5800/api/<span class="has-text-link">&lt;collection&gt;</span>/<span class="has-text-link">&lt;id&gt;</span></p>
                          </code>
                        </div>



                    </div>
                </section>

            </body>
        </html>
        "#,
        data_files.join("")
    );

    res.render(Text::Html(html));
    Ok(())
}

#[handler]
pub async fn delete_collection(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) {
    let app_config = depot.obtain::<AppConfig>().unwrap().clone();
    let data_dir = &app_config.data_dir;
    let file_path = req.param::<String>("f").unwrap();

    delete_collection_sync(data_dir, &file_path).await.unwrap();

    res.render(Redirect::other("/"));
}
