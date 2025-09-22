use crate::error::AppResult;
use crate::utils::delete_collection_sync;
use crate::AppConfig;
use salvo::prelude::*;
use std::path::PathBuf;

#[handler]
pub async fn index(res: &mut Response, depot: &mut Depot) -> AppResult<()> {
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
                let stem_str = stem.to_string_lossy();
                format!(
                    r#"
                    <div class="card mb-3">
                      <div class="card-content">
                        <div class="columns is-vcentered">
                          <div class="column is-6">
                            <span class="is-family-code has-text-link">/api/{stem}</span>
                          </div>
                          <div class="column is-6 has-text-right">
                            <a href="/api/{stem}" target="_blank" class="button is-info is-light">Open</a>
                            <button class="button is-danger is-light js-delete-btn" data-name="{stem}">
                              Delete
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                    "#,
                    stem = stem_str
                )
            })
        })
        .collect();


let html = format!(
    r#"
    <!DOCTYPE html>
    <html>
        <head>
            <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@1.0.4/css/bulma.min.css">
            <script>
              document.addEventListener("DOMContentLoaded", () => {{
                const modal = document.getElementById("delete-modal");
                const modalBg = modal.querySelector(".modal-background");
                const confirmBtn = modal.querySelector(".confirm-btn");
                let currentTarget = "";

                document.querySelectorAll(".js-delete-btn").forEach(btn => {{
                  btn.addEventListener("click", () => {{
                    currentTarget = btn.dataset.name;
                    modal.classList.add("is-active");
                  }});
                }});

                // Cerrar al hacer clic en el fondo
                modalBg.addEventListener("click", () => modal.classList.remove("is-active"));

                // Cerrar al hacer clic en cualquiera de los botones Cancel
                modal.querySelectorAll(".cancel-btn").forEach(btn => {{
                  btn.addEventListener("click", () => modal.classList.remove("is-active"));
                }});

                // Confirmación
                confirmBtn.addEventListener("click", () => {{
                  window.location.href = `/delete-collection/${{currentTarget}}`;
                }});
              }});
            </script>


            <title>Static API</title>
        </head>

        <body>
            <section class="section">
                <div class="container is-max-desktop">
                    <section class="hero is-info mb-5">
                      <div class="hero-body">
                        <p class="title has-text-white">Static-API</p>
                        <p class="subtitle has-text-white">A fake API for testing</p>
                      </div>
                    </section>

                    <p class="has-text-justified">
                      This is a simple application emulating a basic REST API.
                      Each collection is represented as a JSON file in the file system.
                    </p>

                    <h3 class="title is-4 has-text-grey-dark mt-4">Collections</h3>

                    {collections}

                    <p>JSON files are stored in <strong>{data_dir}</strong></p>

                    <h3 class="title is-4 has-text-grey-dark mt-4">Examples</h3>

                    <div class="columns is-multiline">
                      <!-- GET Collection -->
                      <div class="column is-6">
                        <div class="card">
                          <header class="card-header">
                            <p class="card-header-title">Get all items</p>
                          </header>
                          <div class="card-content">
                            <code>curl -X GET http://localhost:5800/api/&lt;collection&gt;</code>
                          </div>
                        </div>
                      </div>

                      <!-- GET with pagination -->
                      <div class="column is-6">
                        <div class="card">
                          <header class="card-header">
                            <p class="card-header-title">Get with skip &amp; limit</p>
                          </header>
                          <div class="card-content">
                            <code>curl -X GET "http://localhost:5800/api/&lt;collection&gt;?skip=10&amp;limit=5"</code>
                          </div>
                        </div>
                      </div>

                      <!-- GET by ID -->
                      <div class="column is-6">
                        <div class="card">
                          <header class="card-header">
                            <p class="card-header-title">Get by ID</p>
                          </header>
                          <div class="card-content">
                            <code>curl -X GET http://localhost:5800/api/&lt;collection&gt;/&lt;id&gt;</code>
                          </div>
                        </div>
                      </div>

                      <!-- POST new item -->
                      <div class="column is-6">
                        <div class="card">
                          <header class="card-header">
                            <p class="card-header-title">Create (POST)</p>
                          </header>
                          <div class="card-content">
                            <code>
                              curl -X POST -H "Content-Type: application/json" \<br>
                              -d '{{"field1":"value1","field2":"value2"}}' \<br>
                              http://localhost:5800/api/&lt;collection&gt;
                            </code>
                          </div>
                        </div>
                      </div>

                      <!-- PUT update -->
                      <div class="column is-6">
                        <div class="card">
                          <header class="card-header">
                            <p class="card-header-title">Update (PUT)</p>
                          </header>
                          <div class="card-content">
                            <code>
                              curl -X PUT -H "Content-Type: application/json" \<br>
                              -d '{{"field1":"new_value1","new_field2":"value2"}}' \<br>
                              http://localhost:5800/api/&lt;collection&gt;/&lt;id&gt;
                            </code>
                          </div>
                        </div>
                      </div>

                      <!-- DELETE by ID -->
                      <div class="column is-6">
                        <div class="card">
                          <header class="card-header">
                            <p class="card-header-title">Delete by ID</p>
                          </header>
                          <div class="card-content">
                            <code>curl -X DELETE http://localhost:5800/api/&lt;collection&gt;/&lt;id&gt;</code>
                          </div>
                        </div>
                      </div>
                    </div>
                </div>
            </section>

            <!-- Modal confirmación -->
            <div id="delete-modal" class="modal">
              <div class="modal-background"></div>
              <div class="modal-card">
                <header class="modal-card-head">
                  <p class="modal-card-title">Confirm delete</p>
                  <button class="delete cancel-btn" aria-label="close"></button>
                </header>
                <section class="modal-card-body">
                  <p>Are you sure you want to delete this collection?</p>
                </section>
                <footer class="modal-card-foot">
                  <button class="button is-danger confirm-btn">Yes, delete</button>
                  <button class="button cancel-btn">Cancel</button>
                </footer>
              </div>
            </div>

        </body>
    </html>
    "#,
    collections = data_files.join("")
);

    res.render(Text::Html(html));
    Ok(())
}

#[handler]
pub async fn delete_collection(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    let app_config = depot.obtain::<AppConfig>().unwrap().clone();
    let data_dir = &app_config.data_dir;
    let file_path = req.param::<String>("f").unwrap();

    delete_collection_sync(data_dir, &file_path).await.unwrap();

    res.render(Redirect::other("/"));
}
