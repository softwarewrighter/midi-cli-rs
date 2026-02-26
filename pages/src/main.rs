mod components;

use components::abc_example_card::AbcExampleCard;
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
    Chiptune,
    Orchestral,
    Show,
    Electronic,
    Euclidean,
    Melodies,
    AbcImport,
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
            Tab::Chiptune => "Chiptune",
            Tab::Orchestral => "Orchestral",
            Tab::Show => "Show",
            Tab::Electronic => "Electronic",
            Tab::Euclidean => "Euclidean",
            Tab::Melodies => "Melodies",
            Tab::AbcImport => "ABC Import",
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
            Tab::Chiptune,
            Tab::Orchestral,
            Tab::Show,
            Tab::Electronic,
            Tab::Euclidean,
            Tab::Melodies,
            Tab::AbcImport,
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
                    <h1>{"midi-cli-rs"}</h1>
                    <p class="subtitle">{"AI-ready MIDI generation • Listen to mood presets and copy CLI commands"}</p>
                </header>

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
                    {"© 2026 Michael A Wright • MIT License • "}
                    <a href="https://github.com/softwarewrighter/midi-cli-rs" target="_blank">{"GitHub"}</a>
                </p>
                <p class="build-info">
                    {"Built with Rust + FluidSynth • Demo generated with seed-based melodic variation"}
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
        Tab::Chiptune => html! {
            <>
                <p class="seed-note">{"8-bit video game style music. Each seed produces dramatically different output:"}</p>
                <ExampleCard
                    title="Chiptune - Seed 1"
                    description="Square wave leads with synth bass."
                    command="midi-cli-rs preset -m chiptune -d 5 --seed 1 -o output.wav"
                    audio_src="audio/chiptune-1.wav"
                    params={vec![("Seed", "1")]}
                />
                <ExampleCard
                    title="Chiptune - Seed 2"
                    description="Different scale, rhythm, and layer combination."
                    command="midi-cli-rs preset -m chiptune -d 5 --seed 2 -o output.wav"
                    audio_src="audio/chiptune-2.wav"
                    params={vec![("Seed", "2")]}
                />
                <ExampleCard
                    title="Chiptune - Seed 3"
                    description="Unique melodic pattern with percussion."
                    command="midi-cli-rs preset -m chiptune -d 5 --seed 3 -o output.wav"
                    audio_src="audio/chiptune-3.wav"
                    params={vec![("Seed", "3")]}
                />
            </>
        },
        Tab::Orchestral => html! {
            <>
                <p class="seed-note">{"Cinematic orchestral arrangements with strings, brass, and woodwinds:"}</p>
                <ExampleCard
                    title="Orchestral - Seed 1"
                    description="Full orchestral texture with layered instruments."
                    command="midi-cli-rs preset -m orchestral -d 5 --seed 1 -o output.wav"
                    audio_src="audio/orchestral-1.wav"
                    params={vec![("Seed", "1")]}
                />
                <ExampleCard
                    title="Orchestral - Seed 2"
                    description="Different voicing and instrument balance."
                    command="midi-cli-rs preset -m orchestral -d 5 --seed 2 -o output.wav"
                    audio_src="audio/orchestral-2.wav"
                    params={vec![("Seed", "2")]}
                />
                <ExampleCard
                    title="Orchestral - Seed 3"
                    description="Varied melodic contours and dynamics."
                    command="midi-cli-rs preset -m orchestral -d 5 --seed 3 -o output.wav"
                    audio_src="audio/orchestral-3.wav"
                    params={vec![("Seed", "3")]}
                />
            </>
        },
        Tab::Show => html! {
            <>
                <p class="seed-note">{"Broadway/musical theater style with piano, brass, and strings:"}</p>
                <ExampleCard
                    title="Show - Seed 1"
                    description="Theatrical fanfare with big band feel."
                    command="midi-cli-rs preset -m show -d 5 --seed 1 -o output.wav"
                    audio_src="audio/show-1.wav"
                    params={vec![("Seed", "1")]}
                />
                <ExampleCard
                    title="Show - Seed 2"
                    description="Different brass voicings and rhythms."
                    command="midi-cli-rs preset -m show -d 5 --seed 2 -o output.wav"
                    audio_src="audio/show-2.wav"
                    params={vec![("Seed", "2")]}
                />
                <ExampleCard
                    title="Show - Seed 3"
                    description="Unique melodic flourishes and dynamics."
                    command="midi-cli-rs preset -m show -d 5 --seed 3 -o output.wav"
                    audio_src="audio/show-3.wav"
                    params={vec![("Seed", "3")]}
                />
            </>
        },
        Tab::Electronic => html! {
            <>
                <p class="seed-note">{"Plugin moods from the electronic pack (~/.midi-cli-rs/moods/):"}</p>
                <h3 class="subsection">{"8bit (via chiptune)"}</h3>
                <ExampleCard
                    title="8bit - Seed 1"
                    description="Retro video game music with square waves."
                    command="midi-cli-rs preset -m 8bit -d 5 --seed 1 -o output.wav"
                    audio_src="audio/8bit-1.wav"
                    params={vec![("Seed", "1"), ("Plugin", "electronic")]}
                />
                <ExampleCard
                    title="8bit - Seed 2"
                    description="Different melodic pattern and layers."
                    command="midi-cli-rs preset -m 8bit -d 5 --seed 2 -o output.wav"
                    audio_src="audio/8bit-2.wav"
                    params={vec![("Seed", "2"), ("Plugin", "electronic")]}
                />
                <h3 class="subsection">{"Synthwave (via upbeat)"}</h3>
                <ExampleCard
                    title="Synthwave - Seed 1"
                    description="80s retro synth vibes in A minor."
                    command="midi-cli-rs preset -m synthwave -d 5 --seed 1 -o output.wav"
                    audio_src="audio/synthwave-1.wav"
                    params={vec![("Seed", "1"), ("Plugin", "electronic")]}
                />
                <ExampleCard
                    title="Synthwave - Seed 2"
                    description="Different rhythm and bass patterns."
                    command="midi-cli-rs preset -m synthwave -d 5 --seed 2 -o output.wav"
                    audio_src="audio/synthwave-2.wav"
                    params={vec![("Seed", "2"), ("Plugin", "electronic")]}
                />
                <h3 class="subsection">{"Techno (via upbeat)"}</h3>
                <ExampleCard
                    title="Techno - Seed 1"
                    description="High-energy electronic beats at 130 BPM."
                    command="midi-cli-rs preset -m techno -d 5 --seed 1 -o output.wav"
                    audio_src="audio/techno-1.wav"
                    params={vec![("Seed", "1"), ("Plugin", "electronic")]}
                />
                <ExampleCard
                    title="Techno - Seed 2"
                    description="Different synth patterns and intensity."
                    command="midi-cli-rs preset -m techno -d 5 --seed 2 -o output.wav"
                    audio_src="audio/techno-2.wav"
                    params={vec![("Seed", "2"), ("Plugin", "electronic")]}
                />
                <h3 class="subsection">{"Chillout (via ambient)"}</h3>
                <ExampleCard
                    title="Chillout - Seed 1"
                    description="Relaxing downtempo ambient at 85 BPM."
                    command="midi-cli-rs preset -m chillout -d 5 --seed 1 -o output.wav"
                    audio_src="audio/chillout-1.wav"
                    params={vec![("Seed", "1"), ("Plugin", "electronic")]}
                />
                <ExampleCard
                    title="Chillout - Seed 2"
                    description="Different textures and pad movements."
                    command="midi-cli-rs preset -m chillout -d 5 --seed 2 -o output.wav"
                    audio_src="audio/chillout-2.wav"
                    params={vec![("Seed", "2"), ("Plugin", "electronic")]}
                />
            </>
        },
        Tab::Euclidean => html! {
            <>
                <p class="seed-note">{"Native plugin using Bjorklund's algorithm for polyrhythmic Euclidean patterns:"}</p>
                <ExampleCard
                    title="Euclidean - Seed 1"
                    description="Polyrhythmic patterns with bass and piano layers."
                    command="midi-cli-rs preset -m euclidean -d 8 --seed 1 --intensity 70 -o output.wav"
                    audio_src="audio/euclidean-1.wav"
                    params={vec![("Seed", "1"), ("Plugin", "native")]}
                />
                <ExampleCard
                    title="Euclidean - Seed 2"
                    description="Different pulse distributions and rhythmic feel."
                    command="midi-cli-rs preset -m euclidean -d 8 --seed 2 --intensity 70 -o output.wav"
                    audio_src="audio/euclidean-2.wav"
                    params={vec![("Seed", "2"), ("Plugin", "native")]}
                />
                <ExampleCard
                    title="Euclidean - Seed 3"
                    description="Unique interlocking patterns across layers."
                    command="midi-cli-rs preset -m euclidean -d 8 --seed 3 --intensity 70 -o output.wav"
                    audio_src="audio/euclidean-3.wav"
                    params={vec![("Seed", "3"), ("Plugin", "native")]}
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
        Tab::AbcImport => html! {
            <>
                <p class="seed-note">{"Import melodies from ABC notation - a simple text format used by folk music archives:"}</p>
                <AbcExampleCard
                    title="Twinkle Twinkle Little Star"
                    description="Classic nursery rhyme in C major."
                    abc_source={"X:1\nT:Twinkle Twinkle Little Star\nM:4/4\nL:1/4\nQ:1/4=100\nK:C\nC C G G | A A G2 | F F E E | D D C2 |\nG G F F | E E D2 | G G F F | E E D2 |\nC C G G | A A G2 | F F E E | D D C2 |"}
                    command="midi-cli-rs import abc pages/abc/twinkle.abc -o output.wav"
                    audio_src="audio/abc-twinkle.wav"
                    params={vec![("Key", "C"), ("Tempo", "100 BPM")]}
                />
                <AbcExampleCard
                    title="Happy Birthday"
                    description="The traditional birthday song in C major."
                    abc_source={"X:1\nT:Happy Birthday\nM:3/4\nL:1/8\nQ:1/4=120\nK:C\nG2 G2 A2 | G2 c2 B4 | G2 G2 A2 | G2 d2 c4 |\nG2 G2 g2 | e2 c2 B2 A2 | f2 f2 e2 | c2 d2 c4 |"}
                    command="midi-cli-rs import abc pages/abc/happy_birthday.abc -o output.wav"
                    audio_src="audio/abc-happy-birthday.wav"
                    params={vec![("Key", "C"), ("Tempo", "120 BPM")]}
                />
                <AbcExampleCard
                    title="Jingle Bells"
                    description="Classic holiday tune in G major."
                    abc_source={"X:1\nT:Jingle Bells\nM:4/4\nL:1/4\nQ:1/4=140\nK:G\nD B A G | D2 D2 | D B A G | E4 |\nE c B A | F4 | d d c A | B4 |\nD B A G | D2 D2 | D B A G | E E E E |\nE c B A | d d d d | e d c A | G4 |"}
                    command="midi-cli-rs import abc pages/abc/jingle_bells.abc -o output.wav"
                    audio_src="audio/abc-jingle-bells.wav"
                    params={vec![("Key", "G"), ("Tempo", "140 BPM")]}
                />
                <AbcExampleCard
                    title="Ode to Joy"
                    description="Beethoven's iconic theme from Symphony No. 9."
                    abc_source={"X:1\nT:Ode to Joy\nC:Beethoven\nM:4/4\nL:1/4\nQ:1/4=120\nK:C\nE E F G | G F E D | C C D E | E3/2 D/ D2 |\nE E F G | G F E D | C C D E | D3/2 C/ C2 |\nD D E C | D E/2F/2 E C | D E/2F/2 E D | C D G,2 |\nE E F G | G F E D | C C D E | D3/2 C/ C2 |"}
                    command="midi-cli-rs import abc pages/abc/ode_to_joy.abc -o output.wav"
                    audio_src="audio/abc-ode-to-joy.wav"
                    params={vec![("Key", "C"), ("Tempo", "120 BPM"), ("Composer", "Beethoven")]}
                />
                <AbcExampleCard
                    title="Greensleeves"
                    description="Traditional English folk song in A minor."
                    abc_source={"X:1\nT:Greensleeves\nM:6/8\nL:1/8\nQ:3/8=60\nK:Am\nA2 | c3 d e/2f/2 | e3 d c | B3 G A/2B/2 | c3 A A |\nA3 ^G A | B3 ^G E | A3 d e/2f/2 | e3 d c |\nB3 G A/2B/2 | c B A ^G A | B ^G E2 | A4 ||"}
                    command="midi-cli-rs import abc pages/abc/greensleeves.abc -o output.wav"
                    audio_src="audio/abc-greensleeves.wav"
                    params={vec![("Key", "Am"), ("Tempo", "60 BPM")]}
                />
            </>
        },
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
