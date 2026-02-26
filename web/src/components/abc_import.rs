//! ABC notation import component.

use crate::api::{AbcImportRequest, SavedMelody};
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AbcImportProps {
    pub on_import: Callback<AbcImportRequest>,
    pub on_imported: Callback<SavedMelody>,
    #[prop_or_default]
    pub importing: bool,
    #[prop_or_default]
    pub error: Option<String>,
}

#[function_component(AbcImport)]
pub fn abc_import(props: &AbcImportProps) -> Html {
    let abc_content = use_state(String::new);
    let name = use_state(String::new);
    let instrument = use_state(|| "piano".to_string());
    let tempo = use_state(|| "".to_string());
    let collapsed = use_state(|| true);

    let on_abc_input = {
        let abc_content = abc_content.clone();
        Callback::from(move |e: InputEvent| {
            let target: HtmlTextAreaElement = e.target_unchecked_into();
            abc_content.set(target.value());
        })
    };

    let on_name_input = {
        let name = name.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            name.set(input.value());
        })
    };

    let on_instrument_change = {
        let instrument = instrument.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            instrument.set(select.value());
        })
    };

    let on_tempo_input = {
        let tempo = tempo.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            tempo.set(input.value());
        })
    };

    let on_submit = {
        let abc_content = abc_content.clone();
        let name = name.clone();
        let instrument = instrument.clone();
        let tempo = tempo.clone();
        let on_import = props.on_import.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let req = AbcImportRequest {
                abc_content: (*abc_content).clone(),
                name: if name.is_empty() {
                    None
                } else {
                    Some((*name).clone())
                },
                instrument: Some((*instrument).clone()),
                tempo: tempo.parse().ok(),
            };

            on_import.emit(req);
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
                <form onsubmit={on_submit}>
                    <div class="form-group">
                        <label for="abc-content">{"ABC Notation"}</label>
                        <textarea
                            id="abc-content"
                            placeholder="X:1\nT:My Tune\nM:4/4\nL:1/4\nK:C\nCDEF|GABC|"
                            value={(*abc_content).clone()}
                            oninput={on_abc_input}
                            rows="8"
                            class="abc-textarea"
                            required=true
                        />
                    </div>

                    <div class="form-row">
                        <div class="form-group">
                            <label for="melody-name">{"Name (optional)"}</label>
                            <input
                                type="text"
                                id="melody-name"
                                placeholder="Uses T: field if empty"
                                value={(*name).clone()}
                                oninput={on_name_input}
                            />
                        </div>

                        <div class="form-group">
                            <label for="instrument">{"Instrument"}</label>
                            <select
                                id="instrument"
                                value={(*instrument).clone()}
                                onchange={on_instrument_change}
                            >
                                <option value="piano">{"Piano"}</option>
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
                                type="number"
                                id="tempo"
                                placeholder="Uses Q: or 120"
                                value={(*tempo).clone()}
                                oninput={on_tempo_input}
                                min="40"
                                max="240"
                            />
                        </div>
                    </div>

                    { if let Some(ref err) = props.error {
                        html! { <div class="import-error">{err}</div> }
                    } else {
                        html! {}
                    }}

                    <button
                        type="submit"
                        class="btn-primary"
                        disabled={props.importing || abc_content.is_empty()}
                    >
                        { if props.importing {
                            html! { <span class="loading"></span> }
                        } else {
                            html! { "Import" }
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
