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
            <ExampleCard
                title="Suspense Mood"
                description="Tense, atmospheric music perfect for thriller scenes or building anticipation."
                command="midi-cli-rs preset -m suspense -d 5 --seed 20260220 -k Am --intensity 60 -t 70 -o output.wav"
                audio_src="audio/suspense-demo.wav"
                params={vec![
                    ("Mood", "suspense"),
                    ("Duration", "5s"),
                    ("Key", "A minor"),
                    ("Intensity", "60%"),
                    ("Tempo", "70 BPM"),
                ]}
            />
        },
        Tab::Eerie => html! {
            <ExampleCard
                title="Eerie Mood"
                description="Unsettling, mysterious music for horror or supernatural themes."
                command="midi-cli-rs preset -m eerie -d 5 --seed 20260220 -k Dm --intensity 50 -t 60 -o output.wav"
                audio_src="audio/eerie-demo.wav"
                params={vec![
                    ("Mood", "eerie"),
                    ("Duration", "5s"),
                    ("Key", "D minor"),
                    ("Intensity", "50%"),
                    ("Tempo", "60 BPM"),
                ]}
            />
        },
        Tab::Upbeat => html! {
            <ExampleCard
                title="Upbeat Mood"
                description="Energetic, positive music for celebrations or exciting moments."
                command="midi-cli-rs preset -m upbeat -d 5 --seed 20260220 -k C --intensity 70 -t 120 -o output.wav"
                audio_src="audio/upbeat-demo.wav"
                params={vec![
                    ("Mood", "upbeat"),
                    ("Duration", "5s"),
                    ("Key", "C major"),
                    ("Intensity", "70%"),
                    ("Tempo", "120 BPM"),
                ]}
            />
        },
        Tab::Calm => html! {
            <ExampleCard
                title="Calm Mood"
                description="Peaceful, relaxing music for meditation or gentle scenes."
                command="midi-cli-rs preset -m calm -d 5 --seed 20260220 -k G --intensity 30 -t 72 -o output.wav"
                audio_src="audio/calm-demo.wav"
                params={vec![
                    ("Mood", "calm"),
                    ("Duration", "5s"),
                    ("Key", "G major"),
                    ("Intensity", "30%"),
                    ("Tempo", "72 BPM"),
                ]}
            />
        },
        Tab::Ambient => html! {
            <ExampleCard
                title="Ambient Mood"
                description="Ethereal, atmospheric soundscapes for background ambiance."
                command="midi-cli-rs preset -m ambient -d 8 --seed 20260220 -k Em --intensity 40 -t 60 -o output.wav"
                audio_src="audio/ambient-demo.wav"
                params={vec![
                    ("Mood", "ambient"),
                    ("Duration", "8s"),
                    ("Key", "E minor"),
                    ("Intensity", "40%"),
                    ("Tempo", "60 BPM"),
                ]}
            />
        },
        Tab::Jazz => html! {
            <ExampleCard
                title="Jazz Mood"
                description="Smooth, sophisticated jazz with complex harmonies and swing feel."
                command="midi-cli-rs preset -m jazz -d 6 --seed 20260220 -k F --intensity 50 -t 100 -o output.wav"
                audio_src="audio/jazz-demo.wav"
                params={vec![
                    ("Mood", "jazz"),
                    ("Duration", "6s"),
                    ("Key", "F major"),
                    ("Intensity", "50%"),
                    ("Tempo", "100 BPM"),
                ]}
            />
        },
        Tab::Melodies => html! {
            <>
                <ExampleCard
                    title="Piano - Nursery Rhyme"
                    description="A familiar melody inspired by 'Twinkle Twinkle Little Star' (public domain)."
                    command="midi-cli-rs generate --notes \"C4:1:80@0,C4:1:80@1,G4:1:85@2,G4:1:85@3,A4:1:90@4,A4:1:90@5,G4:2:85@6,...\" -i piano -t 110 -o output.wav"
                    audio_src="audio/melody-piano.wav"
                    params={vec![
                        ("Instrument", "piano"),
                        ("Style", "Classical / Nursery"),
                        ("Tempo", "110 BPM"),
                    ]}
                />
                <ExampleCard
                    title="Strings - Lyrical Sweep"
                    description="A sweeping, expressive melody with long sustained notes."
                    command="midi-cli-rs generate --notes \"E4:3:70@0,G4:1:75@3,A4:2:80@4,G4:2:75@6,E4:2:70@8,...\" -i strings -t 66 -o output.wav"
                    audio_src="audio/melody-strings.wav"
                    params={vec![
                        ("Instrument", "strings"),
                        ("Style", "Romantic / Cinematic"),
                        ("Tempo", "66 BPM"),
                    ]}
                />
                <ExampleCard
                    title="Bass - Walking Groove"
                    description="A funky walking bass line with rhythmic drive."
                    command="midi-cli-rs generate --notes \"E2:1:90@0,G2:1:85@1,A2:1:90@2,B2:1:85@3,C3:1:95@4,...\" -i bass -t 128 -o output.wav"
                    audio_src="audio/melody-bass.wav"
                    params={vec![
                        ("Instrument", "bass"),
                        ("Style", "Funk / Jazz"),
                        ("Tempo", "128 BPM"),
                    ]}
                />
                <ExampleCard
                    title="Cello - Deep Expression"
                    description="A slow, emotional melody showcasing the cello's rich low register."
                    command="midi-cli-rs generate --notes \"G2:4:75@0,B2:2:80@4,D3:2:85@6,G3:4:90@8,...\" -i cello -t 54 -o output.wav"
                    audio_src="audio/melody-cello.wav"
                    params={vec![
                        ("Instrument", "cello"),
                        ("Style", "Classical / Emotional"),
                        ("Tempo", "54 BPM"),
                    ]}
                />
            </>
        },
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
