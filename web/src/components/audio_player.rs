//! Audio player component for playing generated WAV files.

use yew::prelude::*;

/// Props for the AudioPlayer component.
#[derive(Properties, PartialEq)]
pub struct AudioPlayerProps {
    /// URL of the audio file to play.
    pub src: String,
}

/// Audio player widget wrapping an HTML5 audio element.
#[function_component(AudioPlayer)]
pub fn audio_player(props: &AudioPlayerProps) -> Html {
    html! {
        <div class="audio-player">
            <audio controls=true autoplay=false src={props.src.clone()}>
                {"Your browser does not support the audio element."}
            </audio>
        </div>
    }
}
