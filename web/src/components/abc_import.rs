//! ABC notation import component.

use crate::api::AbcImportRequest;
use gloo_file::callbacks::FileReader;
use gloo_file::File;
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AbcImportProps {
    pub on_import: Callback<AbcImportRequest>,
    #[prop_or_default]
    pub importing: bool,
    #[prop_or_default]
    pub error: Option<String>,
    #[prop_or_default]
    pub success: Option<String>,
}

#[function_component(AbcImport)]
pub fn abc_import(props: &AbcImportProps) -> Html {
    let collapsed = use_state(|| false);
    let file_reader = use_state(|| None::<FileReader>);

    // Use refs to read values directly from DOM (more reliable than controlled inputs in WASM)
    let textarea_ref = use_node_ref();
    let name_ref = use_node_ref();
    let instrument_ref = use_node_ref();
    let tempo_ref = use_node_ref();

    let on_file_upload = {
        let textarea_ref = textarea_ref.clone();
        let file_reader = file_reader.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let file = File::from(file);
                    let textarea_ref = textarea_ref.clone();
                    let reader = gloo_file::callbacks::read_as_text(&file, move |result| {
                        if let Ok(content) = result {
                            if let Some(textarea) = textarea_ref.cast::<HtmlTextAreaElement>() {
                                textarea.set_value(&content);
                            }
                        }
                    });
                    file_reader.set(Some(reader));
                }
            }
        })
    };

    let on_submit = {
        let textarea_ref = textarea_ref.clone();
        let name_ref = name_ref.clone();
        let instrument_ref = instrument_ref.clone();
        let tempo_ref = tempo_ref.clone();
        let on_import = props.on_import.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            // Read values directly from DOM
            let content = textarea_ref
                .cast::<HtmlTextAreaElement>()
                .map(|t| t.value())
                .unwrap_or_default();

            if content.trim().is_empty() {
                web_sys::console::log_1(&"ABC Import: content is empty".into());
                return;
            }

            let name = name_ref
                .cast::<HtmlInputElement>()
                .map(|i| i.value())
                .filter(|s| !s.is_empty());

            let instrument = instrument_ref
                .cast::<HtmlSelectElement>()
                .map(|s| s.value());

            let tempo = tempo_ref
                .cast::<HtmlInputElement>()
                .and_then(|i| i.value().parse().ok());

            web_sys::console::log_1(&format!("ABC Import: submitting, content length: {}", content.len()).into());

            let req = AbcImportRequest {
                abc_content: content,
                name,
                instrument,
                tempo,
            };

            on_import.emit(req);

            // Clear the textarea after submit
            if let Some(textarea) = textarea_ref.cast::<HtmlTextAreaElement>() {
                textarea.set_value("");
            }
        })
    };

    let toggle_collapsed = {
        let collapsed = collapsed.clone();
        Callback::from(move |_| {
            collapsed.set(!*collapsed);
        })
    };

    html! {
        <div class="card abc-import">
            <h2 onclick={toggle_collapsed.clone()} style="cursor: pointer;">
                {"Import ABC Notation "}
                <span class="collapse-indicator">{ if *collapsed { "+" } else { "-" } }</span>
            </h2>

            if !*collapsed {
                // Success message
                if let Some(ref msg) = props.success {
                    <div class="import-success" style="background: #2d5a2d; padding: 10px; margin-bottom: 15px; border-radius: 4px;">
                        {msg}
                        {" - Scroll down to 'Saved Melodies'"}
                    </div>
                }

                // Error message
                if let Some(ref err) = props.error {
                    <div class="import-error" style="background: #5a2d2d; padding: 10px; margin-bottom: 15px; border-radius: 4px;">
                        {err}
                    </div>
                }

                <form onsubmit={on_submit}>
                    // File upload
                    <div class="form-group">
                        <label for="abc-file">{"Upload .abc file"}</label>
                        <input
                            type="file"
                            id="abc-file"
                            accept=".abc,.txt"
                            onchange={on_file_upload}
                            class="file-input"
                        />
                    </div>

                    // Or paste text
                    <div class="form-group">
                        <label for="abc-content">{"Or paste ABC notation"}</label>
                        <textarea
                            ref={textarea_ref}
                            id="abc-content"
                            placeholder={"X:1\nT:My Tune\nM:4/4\nL:1/4\nK:C\nCDEF|GABC|"}
                            rows="8"
                            class="abc-textarea"
                        />
                    </div>

                    <div class="form-row">
                        <div class="form-group">
                            <label for="melody-name">{"Name (optional)"}</label>
                            <input
                                ref={name_ref}
                                type="text"
                                id="melody-name"
                                placeholder="Uses T: field if empty"
                            />
                        </div>

                        <div class="form-group">
                            <label for="instrument">{"Instrument"}</label>
                            <select
                                ref={instrument_ref}
                                id="instrument"
                            >
                                <option value="piano" selected=true>{"Piano"}</option>
                                <option value="strings">{"Strings"}</option>
                                <option value="guitar">{"Guitar"}</option>
                                <option value="flute">{"Flute"}</option>
                                <option value="violin">{"Violin"}</option>
                                <option value="cello">{"Cello"}</option>
                                <option value="bass">{"Bass"}</option>
                                <option value="organ">{"Organ"}</option>
                            </select>
                        </div>

                        <div class="form-group">
                            <label for="tempo">{"Tempo (optional)"}</label>
                            <input
                                ref={tempo_ref}
                                type="number"
                                id="tempo"
                                placeholder="Uses Q: or 120"
                                min="40"
                                max="240"
                            />
                        </div>
                    </div>

                    <button
                        type="submit"
                        class="btn-primary"
                        disabled={props.importing}
                    >
                        { if props.importing {
                            html! { <><span class="loading"></span>{" Importing..."}</> }
                        } else {
                            html! { "Import ABC" }
                        }}
                    </button>
                </form>

                <div class="abc-help">
                    <details>
                        <summary>{"ABC Notation Help"}</summary>
                        <pre class="abc-example">{r#"X:1             (tune number)
T:Twinkle Star  (title)
M:4/4           (time signature)
L:1/4           (default note length)
Q:1/4=100       (tempo)
K:C             (key)
C C G G | A A G2 |"#}</pre>
                        <p>{"Notes: C D E F G A B (uppercase=octave 4, lowercase=octave 5)"}</p>
                        <p>{"Durations: C2=half, C/2=eighth, C3/2=dotted quarter"}</p>
                        <p>{"Rests: z (same duration rules)"}</p>
                    </details>
                </div>
            }
        </div>
    }
}
