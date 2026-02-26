//! ABC notation example card component for the demo page.

use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AbcExampleCardProps {
    pub title: AttrValue,
    pub description: AttrValue,
    pub abc_source: AttrValue,
    pub command: AttrValue,
    pub audio_src: AttrValue,
    #[prop_or_default]
    pub params: Vec<(&'static str, &'static str)>,
}

#[function_component(AbcExampleCard)]
pub fn abc_example_card(props: &AbcExampleCardProps) -> Html {
    let copied = use_state(|| false);
    let abc_copied = use_state(|| false);
    let show_abc = use_state(|| false);

    let on_copy_command = {
        let command = props.command.clone();
        let copied = copied.clone();
        Callback::from(move |_| {
            let command = command.clone();
            let copied = copied.clone();
            spawn_local(async move {
                if let Some(window) = window() {
                    let clipboard = window.navigator().clipboard();
                    let _ = clipboard.write_text(&command);
                    copied.set(true);
                    gloo_timers::callback::Timeout::new(2000, move || {
                        copied.set(false);
                    })
                    .forget();
                }
            });
        })
    };

    let on_copy_abc = {
        let abc_source = props.abc_source.clone();
        let abc_copied = abc_copied.clone();
        Callback::from(move |_| {
            let abc_source = abc_source.clone();
            let abc_copied = abc_copied.clone();
            spawn_local(async move {
                if let Some(window) = window() {
                    let clipboard = window.navigator().clipboard();
                    let _ = clipboard.write_text(&abc_source);
                    abc_copied.set(true);
                    gloo_timers::callback::Timeout::new(2000, move || {
                        abc_copied.set(false);
                    })
                    .forget();
                }
            });
        })
    };

    let toggle_abc = {
        let show_abc = show_abc.clone();
        Callback::from(move |_| {
            show_abc.set(!*show_abc);
        })
    };

    let copy_btn_class = if *copied {
        "copy-btn copied"
    } else {
        "copy-btn"
    };
    let copy_btn_text = if *copied { "Copied!" } else { "Copy" };

    let abc_copy_btn_class = if *abc_copied {
        "copy-btn copied"
    } else {
        "copy-btn"
    };
    let abc_copy_btn_text = if *abc_copied { "Copied!" } else { "Copy ABC" };

    html! {
        <div class="card abc-card">
            <h3>{&props.title}</h3>
            <p>{&props.description}</p>

            // Parameter badges
            if !props.params.is_empty() {
                <div class="params">
                    {props.params.iter().map(|(key, value)| {
                        html! {
                            <span class="param-badge" key={*key}>
                                <strong>{key}{": "}</strong>{value}
                            </span>
                        }
                    }).collect::<Html>()}
                </div>
            }

            // ABC Source toggle
            <div class="abc-source-section">
                <button class="btn-toggle-abc" onclick={toggle_abc}>
                    { if *show_abc { "Hide ABC Source" } else { "Show ABC Source" } }
                </button>

                if *show_abc {
                    <div class="abc-source-block">
                        <pre class="abc-source">{&props.abc_source}</pre>
                        <button class={abc_copy_btn_class} onclick={on_copy_abc}>{abc_copy_btn_text}</button>
                    </div>
                }
            </div>

            // Command block
            <div class="command-block">
                <code>{&props.command}</code>
                <button class={copy_btn_class} onclick={on_copy_command}>{copy_btn_text}</button>
            </div>

            // Audio player
            <div class="audio-player" key={props.audio_src.to_string()}>
                <audio controls=true preload="metadata" src={props.audio_src.clone()}>
                    {"Your browser does not support the audio element."}
                </audio>
            </div>
        </div>
    }
}
