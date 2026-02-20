mod components;

use components::example_card::ExampleCard;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Suspense,
    Eerie,
    Upbeat,
    Calm,
    Ambient,
    Jazz,
    Melodies,
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            Tab::Suspense => "Suspense",
            Tab::Eerie => "Eerie",
            Tab::Upbeat => "Upbeat",
            Tab::Calm => "Calm",
            Tab::Ambient => "Ambient",
            Tab::Jazz => "Jazz",
            Tab::Melodies => "Melodies",
        }
    }

    fn all() -> &'static [Tab] {
        &[
            Tab::Suspense,
            Tab::Eerie,
            Tab::Upbeat,
            Tab::Calm,
            Tab::Ambient,
            Tab::Jazz,
            Tab::Melodies,
        ]
    }
}

#[function_component(App)]
fn app() -> Html {
    let active_tab = use_state(|| Tab::Suspense);

    let on_tab_click = {
        let active_tab = active_tab.clone();
        Callback::from(move |tab: Tab| {
            active_tab.set(tab);
        })
    };

    html! {
        <>
            // GitHub corner
            <a href="https://github.com/softwarewrighter/midi-cli-rs" class="github-corner" aria-label="View source on GitHub" target="_blank">
                <svg viewBox="0 0 250 250" aria-hidden="true">
                    <path d="M0,0 L115,115 L130,115 L142,142 L250,250 L250,0 Z"></path>
                    <path d="M128.3,109.0 C113.8,99.7 119.0,89.6 119.0,89.6 C122.0,82.7 120.5,78.6 120.5,78.6 C119.2,72.0 123.4,76.3 123.4,76.3 C127.3,80.9 125.5,87.3 125.5,87.3 C122.9,97.6 130.6,101.9 134.4,103.2" fill="currentColor" style="transform-origin: 130px 106px;" class="octo-arm"></path>
                    <path d="M115.0,115.0 C114.9,115.1 118.7,116.5 119.8,115.4 L133.7,101.6 C136.9,99.2 139.9,98.4 142.2,98.6 C133.8,88.0 127.5,74.4 143.8,58.0 C148.5,53.4 154.0,51.2 159.7,51.0 C160.3,49.4 163.2,43.6 171.4,40.1 C171.4,40.1 176.1,42.5 178.8,56.2 C183.1,58.6 187.2,61.8 190.9,65.4 C ## 194.5,69.0 197.7,73.2 200.1,77.6 C ## 213.8,80.2 216.3,84.9 216.3,84.9 C212.7,93.1 206.9,96.0 205.4,96.6 C205.1,102.4 203.0,107.8 198.3,112.5 C181.9,128.9 168.3,122.5 157.7,114.1 C ## 157.9,116.9 156.7,120.9 152.7,124.9 L ## 141.0,136.5 C139.8,137.7 141.6,141.9 141.8,141.8 Z" fill="currentColor" class="octo-body"></path>
                </svg>
            </a>

            <div class="app-container">
                <header>
                    <h1>{"MIDI CLI Demo"}</h1>
                    <p class="subtitle">{"Procedural MIDI music generation for AI coding agents"}</p>
                </header>

                <div class="intro">
                    <h2>{"About"}</h2>
                    <p>
                        {"This demo showcases "}
                        <code>{"midi-cli-rs"}</code>
                        {", a Rust CLI tool for generating MIDI music with mood presets."}
                    </p>
                    <p>
                        {"Each example below shows the exact CLI command used and lets you listen to the generated audio."}
                    </p>
                </div>

                // Tabs
                <div class="tabs">
                    {Tab::all().iter().map(|&tab| {
                        let on_click = {
                            let on_tab_click = on_tab_click.clone();
                            Callback::from(move |_| on_tab_click.emit(tab))
                        };
                        let class = if *active_tab == tab { "tab active" } else { "tab" };
                        html! {
                            <button {class} onclick={on_click} key={tab.label()}>
                                {tab.label()}
                            </button>
                        }
                    }).collect::<Html>()}
                </div>

                // Tab content
                {render_tab_content(*active_tab)}
            </div>

            <footer>
                <p>
                    {"Built with Rust + Yew | "}
                    <a href="https://github.com/softwarewrighter/midi-cli-rs" target="_blank">{"View on GitHub"}</a>
                    {" | MIT License"}
                </p>
            </footer>
        </>
    }
}

fn render_tab_content(tab: Tab) -> Html {
    match tab {
        Tab::Suspense => html! {
            <>
                <p class="seed-note">{"Each seed produces a unique variation. Listen to all three:"}</p>
                <ExampleCard
                    title="Suspense - Seed 1"
                    description="Tense, atmospheric music perfect for thriller scenes."
                    command="midi-cli-rs preset -m suspense -d 5 --seed 1 -o output.wav"
                    audio_src="audio/suspense-1.wav"
                    params={vec![("Seed", "1")]}
                />
                <ExampleCard
                    title="Suspense - Seed 2"
                    description="Different melodic contour and instrument choices."
                    command="midi-cli-rs preset -m suspense -d 5 --seed 2 -o output.wav"
                    audio_src="audio/suspense-2.wav"
                    params={vec![("Seed", "2")]}
                />
                <ExampleCard
                    title="Suspense - Seed 3"
                    description="Another unique variation with different phrasing."
                    command="midi-cli-rs preset -m suspense -d 5 --seed 3 -o output.wav"
                    audio_src="audio/suspense-3.wav"
                    params={vec![("Seed", "3")]}
                />
            </>
        },
        Tab::Eerie => html! {
            <>
                <p class="seed-note">{"Each seed produces a unique variation. Listen to all three:"}</p>
                <ExampleCard
                    title="Eerie - Seed 1"
                    description="Unsettling, mysterious music for horror themes."
                    command="midi-cli-rs preset -m eerie -d 5 --seed 1 -o output.wav"
                    audio_src="audio/eerie-1.wav"
                    params={vec![("Seed", "1")]}
                />
                <ExampleCard
                    title="Eerie - Seed 2"
                    description="Different dissonant patterns and textures."
                    command="midi-cli-rs preset -m eerie -d 5 --seed 2 -o output.wav"
                    audio_src="audio/eerie-2.wav"
                    params={vec![("Seed", "2")]}
                />
                <ExampleCard
                    title="Eerie - Seed 3"
                    description="Unique bell tones and movement."
                    command="midi-cli-rs preset -m eerie -d 5 --seed 3 -o output.wav"
                    audio_src="audio/eerie-3.wav"
                    params={vec![("Seed", "3")]}
                />
            </>
        },
        Tab::Upbeat => html! {
            <>
                <p class="seed-note">{"Each seed produces a unique variation. Listen to all three:"}</p>
                <ExampleCard
                    title="Upbeat - Seed 1"
                    description="Energetic, positive music for celebrations."
                    command="midi-cli-rs preset -m upbeat -d 5 --seed 1 -o output.wav"
                    audio_src="audio/upbeat-1.wav"
                    params={vec![("Seed", "1")]}
                />
                <ExampleCard
                    title="Upbeat - Seed 2"
                    description="Different rhythm pattern and bass line."
                    command="midi-cli-rs preset -m upbeat -d 5 --seed 2 -o output.wav"
                    audio_src="audio/upbeat-2.wav"
                    params={vec![("Seed", "2")]}
                />
                <ExampleCard
                    title="Upbeat - Seed 3"
                    description="Another variation with unique melody hints."
                    command="midi-cli-rs preset -m upbeat -d 5 --seed 3 -o output.wav"
                    audio_src="audio/upbeat-3.wav"
                    params={vec![("Seed", "3")]}
                />
            </>
        },
        Tab::Calm => html! {
            <>
                <p class="seed-note">{"Each seed produces a unique variation. Listen to all three:"}</p>
                <ExampleCard
                    title="Calm - Seed 1"
                    description="Peaceful arpeggios with ascending contour."
                    command="midi-cli-rs preset -m calm -d 5 --seed 1 -o output.wav"
                    audio_src="audio/calm-1.wav"
                    params={vec![("Seed", "1")]}
                />
                <ExampleCard
                    title="Calm - Seed 2"
                    description="Different melodic pattern with phrase inversion."
                    command="midi-cli-rs preset -m calm -d 5 --seed 2 -o output.wav"
                    audio_src="audio/calm-2.wav"
                    params={vec![("Seed", "2")]}
                />
                <ExampleCard
                    title="Calm - Seed 3"
                    description="Varied rhythm and interval leaps."
                    command="midi-cli-rs preset -m calm -d 5 --seed 3 -o output.wav"
                    audio_src="audio/calm-3.wav"
                    params={vec![("Seed", "3")]}
                />
            </>
        },
        Tab::Ambient => html! {
            <>
                <p class="seed-note">{"Each seed produces a unique variation. Listen to all three:"}</p>
                <ExampleCard
                    title="Ambient - Seed 1"
                    description="Ethereal soundscape with sustained drones."
                    command="midi-cli-rs preset -m ambient -d 5 --seed 1 -o output.wav"
                    audio_src="audio/ambient-1.wav"
                    params={vec![("Seed", "1")]}
                />
                <ExampleCard
                    title="Ambient - Seed 2"
                    description="Different texture with sporadic tones."
                    command="midi-cli-rs preset -m ambient -d 5 --seed 2 -o output.wav"
                    audio_src="audio/ambient-2.wav"
                    params={vec![("Seed", "2")]}
                />
                <ExampleCard
                    title="Ambient - Seed 3"
                    description="Unique pulsing patterns and sub-bass."
                    command="midi-cli-rs preset -m ambient -d 5 --seed 3 -o output.wav"
                    audio_src="audio/ambient-3.wav"
                    params={vec![("Seed", "3")]}
                />
            </>
        },
        Tab::Jazz => html! {
            <>
                <p class="seed-note">{"Each seed produces a unique variation. Listen to all three:"}</p>
                <ExampleCard
                    title="Jazz - Seed 1"
                    description="Walking bass with swing feel."
                    command="midi-cli-rs preset -m jazz -d 5 --seed 1 -o output.wav"
                    audio_src="audio/jazz-1.wav"
                    params={vec![("Seed", "1")]}
                />
                <ExampleCard
                    title="Jazz - Seed 2"
                    description="Different voicings and comping patterns."
                    command="midi-cli-rs preset -m jazz -d 5 --seed 2 -o output.wav"
                    audio_src="audio/jazz-2.wav"
                    params={vec![("Seed", "2")]}
                />
                <ExampleCard
                    title="Jazz - Seed 3"
                    description="Unique flourishes and grace notes."
                    command="midi-cli-rs preset -m jazz -d 5 --seed 3 -o output.wav"
                    audio_src="audio/jazz-3.wav"
                    params={vec![("Seed", "3")]}
                />
            </>
        },
        Tab::Melodies => html! {
            <>
                <ExampleCard
                    title="Piano - Nursery Rhyme"
                    description="'Twinkle Twinkle' inspired melody (public domain)."
                    command="midi-cli-rs generate --notes \"C4:1:80@0,C4:1:80@1,G4:1:85@2,G4:1:85@3,A4:1:90@4,A4:1:90@5,G4:2:85@6,F4:1:80@8,F4:1:80@9,E4:1:85@10,E4:1:85@11,D4:1:80@12,D4:1:80@13,C4:2:90@14\" -i piano -t 110 -o output.wav"
                    audio_src="audio/melody-piano.wav"
                    params={vec![("Instrument", "piano"), ("Tempo", "110 BPM")]}
                />
                <ExampleCard
                    title="Strings - Lyrical Sweep"
                    description="Sweeping melody with sustained notes."
                    command="midi-cli-rs generate --notes \"E4:3:70@0,G4:1:75@3,A4:2:80@4,G4:2:75@6,E4:2:70@8,D4:2:75@10,E4:4:80@12,C4:3:70@16,E4:1:75@19,G4:2:80@20,A4:2:85@22,G4:4:80@24\" -i strings -t 66 -o output.wav"
                    audio_src="audio/melody-strings.wav"
                    params={vec![("Instrument", "strings"), ("Tempo", "66 BPM")]}
                />
                <ExampleCard
                    title="Bass - Walking Groove"
                    description="Funky walking bass line."
                    command="midi-cli-rs generate --notes \"E2:1:90@0,G2:1:85@1,A2:1:90@2,B2:1:85@3,C3:1:95@4,B2:1:85@5,A2:1:90@6,G2:1:85@7,E2:1:90@8,D2:1:85@9,C2:1:90@10,D2:1:85@11,E2:2:95@12,G2:1:85@14,E2:1:90@15\" -i bass -t 128 -o output.wav"
                    audio_src="audio/melody-bass.wav"
                    params={vec![("Instrument", "bass"), ("Tempo", "128 BPM")]}
                />
                <ExampleCard
                    title="Cello - Deep Expression"
                    description="Slow, emotional cello melody."
                    command="midi-cli-rs generate --notes \"G2:4:75@0,B2:2:80@4,D3:2:85@6,G3:4:90@8,F#3:2:85@12,E3:2:80@14,D3:4:85@16,C3:2:80@20,B2:2:75@22,G2:6:80@24\" -i cello -t 54 -o output.wav"
                    audio_src="audio/melody-cello.wav"
                    params={vec![("Instrument", "cello"), ("Tempo", "54 BPM")]}
                />
            </>
        },
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
