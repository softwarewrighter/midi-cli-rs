use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ExampleCardProps {
    pub title: AttrValue,
    pub description: AttrValue,
    pub command: AttrValue,
    pub audio_src: AttrValue,
    #[prop_or_default]
    pub params: Vec<(&'static str, &'static str)>,
}

#[function_component(ExampleCard)]
pub fn example_card(props: &ExampleCardProps) -> Html {
    let copied = use_state(|| false);

    let on_copy = {
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
                    // Reset after 2 seconds
                    gloo_timers::callback::Timeout::new(2000, move || {
                        copied.set(false);
                    })
                    .forget();
                }
            });
        })
    };

    let copy_btn_class = if *copied {
        "copy-btn copied"
    } else {
        "copy-btn"
    };
    let copy_btn_text = if *copied { "Copied!" } else { "Copy" };

    html! {
        <div class="card">
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

            // Command block
            <div class="command-block">
                <code>{&props.command}</code>
                <button class={copy_btn_class} onclick={on_copy}>{copy_btn_text}</button>
            </div>

            // Audio player - key forces recreation when src changes
            <div class="audio-player" key={props.audio_src.to_string()}>
                <audio controls=true preload="metadata" src={props.audio_src.clone()}>
                    {"Your browser does not support the audio element."}
                </audio>
            </div>
        </div>
    }
}
