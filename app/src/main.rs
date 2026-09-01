#![allow(non_snake_case)]
use dioxus::prelude::*;

mod demos;

const STYLE: Asset = asset!("/assets/style.css");
const PFP: Asset = asset!("/assets/pfp.jpg");
const FAVICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'%3E%3Crect width='100' height='100' rx='22' fill='%230071e3'/%3E%3Ctext x='50' y='69' font-family='-apple-system,Segoe UI,sans-serif' font-size='58' font-weight='600' fill='white' text-anchor='middle'%3EE%3C/text%3E%3C/svg%3E";

fn main() {
    dioxus::launch(App);
}

#[component]
fn Work(no: &'static str, title: String, desc: String, meta: Element, href: &'static str) -> Element {
    rsx! {
        a { class: "work", href: "{href}",
            div { class: "no", "{no}" }
            div { class: "body",
                h3 { "{title}" }
                div { class: "desc", "{desc}" }
                div { class: "arrow", "View source →" }
            }
            div { class: "meta", {meta} }
        }
    }
}

#[component]
fn App() -> Element {
    use_effect(|| {
        demos::setup_all();
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: STYLE }

        button { class: "toggle", id: "toggle", "Dark" }

        div { class: "wrap",

            header { class: "hero",
                img { class: "avatar", src: PFP, alt: "Elijah Couch", width: "104", height: "104" }
                div { class: "eyebrow", "Selected Works" }
                h1 { "Elijah\u{00A0}Couch" }
                p { class: "tag",
                    "Rust engineer building cross-platform systems, real-time graphics, and low-level tooling — from a GPU renderer to a Bluetooth stack."
                }
            }

            section { class: "about",
                div { class: "label", span { "About" } span { class: "rule" } }
                p { "I build software close to the metal — mostly in Rust." }
                p {
                    "My work spans two studios: "
                    strong { "Ramp\u{00A0}Stack" }
                    ", a cross-platform Rust application and graphics stack, and "
                    strong { "Space\u{00A0}Soup\u{00A0}VR" }
                    ", a WGPU-powered 3D engine that runs on the Meta\u{00A0}Quest. I like the layer where design meets systems — clean abstractions, real-time rendering, talking to hardware directly, and tools other people build on. A few things below are interactive."
                }
            }

            section { class: "works",
                div { class: "label", span { "Projects" } span { class: "rule" } }

                Work {
                    no: "01", href: "https://github.com/ramp-stack/maverick_os/tree/ble",
                    title: "Bluetooth LE stack".to_string(),
                    desc: "A from-scratch Bluetooth Low Energy stack in Rust, talking to Apple's CoreBluetooth directly through objc2 FFI. Central and peripheral roles — scanning devices (name, RSSI, services) and advertising custom GATT characteristics — plus device-to-device sharing on macOS and iOS.".to_string(),
                    meta: rsx! { "RUST · FFI" br {} "BLE" }
                }
                Work {
                    no: "02", href: "https://github.com/ramp-stack/wgpu_canvas",
                    title: "wgpu_canvas".to_string(),
                    desc: "A pure-Rust GPU canvas built directly on wgpu (WebGPU) — shapes, textures, and cached text with zero JavaScript. Runs on desktop and the web.".to_string(),
                    meta: rsx! { "RUST" br {} span { class: "star", "★ 7" } }
                }
                Work {
                    no: "03", href: "https://github.com/ramp-stack/quartz",
                    title: "Quartz".to_string(),
                    desc: "A 2D game engine written in Rust — the graphics framework behind the Ramp stack, and what the rampstudio editor is built on.".to_string(),
                    meta: rsx! { "RUST" br {} span { class: "star", "★ 3" } }
                }
                Work {
                    no: "04", href: "https://github.com/SpaceSoupVR/space_soup",
                    title: "Space Soup".to_string(),
                    desc: "A 3D and 2D rendering engine powered by WGPU, targeting the Meta Quest through OpenXR and Vulkan — with a scene editor and its own engine layer.".to_string(),
                    meta: rsx! { "RUST · VR" }
                }
                Work {
                    no: "05", href: "https://github.com/ramp-stack/air",
                    title: "air".to_string(),
                    desc: "A decentralized, end-to-end encrypted messaging and identity protocol in async Rust — secp256k1 keys, ChaCha20-Poly1305 encryption, and a name system, running over Tokio and WebSockets.".to_string(),
                    meta: rsx! { "RUST · CRYPTO" }
                }
                Work {
                    no: "06", href: "https://github.com/ramp-stack/maverick_os",
                    title: "maverick_os".to_string(),
                    desc: "The cross-platform layer beneath the Ramp stack — one Rust API over camera, haptics, push notifications, clipboard, sharing, and Bluetooth, across macOS, iOS, and Android.".to_string(),
                    meta: rsx! { "RUST · SYSTEMS" }
                }
                Work {
                    no: "07", href: "https://github.com/ramp-stack/pelican_ui",
                    title: "pelican_ui · prism".to_string(),
                    desc: "A cross-platform UI stack in Rust — prism supplies the low-level layout, rendering, and event primitives on top of wgpu_canvas; pelican_ui is the design system built on it.".to_string(),
                    meta: rsx! { "RUST · UI" }
                }

                div { class: "also",
                    "Also — "
                    a { href: "https://github.com/ElijahNCouch/system18", "system18" }
                    " (a 24/7 trading bot), "
                    a { href: "https://github.com/ElijahNCouch/bevy_qrcode", "bevy_qrcode" }
                    ", "
                    a { href: "https://github.com/ElijahNCouch/sorting_music", "sorting_music" }
                    ", "
                    a { href: "https://github.com/ElijahNCouch/spritesheet_gif_converter", "spritesheet_gif_converter" }
                    ", and more on "
                    a { href: "https://github.com/ElijahNCouch", "GitHub" }
                    "."
                }
            }

            section { class: "playground",
                div { class: "label", span { "Playground — a few things running live" } span { class: "rule" } }
                div { class: "dgrid",

                    div { class: "demo wide",
                        div { class: "dhead", h4 { "Nearby devices" } span { class: "dtag", "maverick_os · ble" } }
                        div { class: "ble-grid",
                            canvas { id: "bleRadar" }
                            div { class: "ble-list", id: "bleList",
                                div { class: "ble-empty", "Press Scan to discover" br {} "nearby BLE devices" }
                            }
                        }
                        div { class: "controls",
                            button { class: "btn pri", id: "bleScan", "Scan" }
                            button { class: "btn ghost", id: "bleClear", "Clear" }
                        }
                        p { class: "cap",
                            "A simulation of the BLE central scanner — it discovers devices with a name, RSSI (signal strength, in dBm) and services; tap one to see its GATT characteristics. The real stack does this from Rust through CoreBluetooth. "
                            a { href: "https://github.com/ramp-stack/maverick_os/tree/ble", "Source →" }
                        }
                    }

                    div { class: "demo",
                        div { class: "dhead", h4 { "Sorting, out loud" } span { class: "dtag", "sorting_music" } }
                        canvas { id: "sortCv" }
                        div { class: "controls",
                            button { class: "btn pri", id: "sortRun", "Sort" }
                            button { class: "btn ghost", id: "sortShuffle", "Shuffle" }
                            select { class: "btn", id: "sortAlgo",
                                option { value: "bubble", "Bubble" }
                                option { value: "insertion", "Insertion" }
                                option { value: "selection", "Selection" }
                            }
                            button { class: "btn ghost", id: "sortSound", "Sound: on" }
                        }
                        p { class: "cap",
                            "A rebuild of my Python sorting visualizer — bars animate as they sort and each comparison plays a tone. "
                            a { href: "https://github.com/ElijahNCouch/sorting_music", "Source →" }
                        }
                    }

                    div { class: "demo",
                        div { class: "dhead", h4 { "Encrypted channel" } span { class: "dtag", "air" } }
                        input { id: "encIn", spellcheck: "false", "aria-label": "Message to encrypt" }
                        div { class: "enc-out", id: "encOut" }
                        div { class: "enc-key", "key\u{00A0} ", b { id: "encKey" } }
                        div { class: "controls", button { class: "btn ghost", id: "encRotate", "Rotate key" } }
                        p { class: "cap",
                            "air secures messages with secp256k1 keys and ChaCha20-Poly1305. This is an illustrative keystream — type and watch it encrypt. "
                            a { href: "https://github.com/ramp-stack/air", "Source →" }
                        }
                    }
                }
            }

            section { class: "collections",
                div { class: "label", span { "Studios" } span { class: "rule" } }
                div { class: "cols",
                    div { class: "col",
                        div { class: "cen", "Ramp Stack" }
                        div { class: "csub", "github.com/ramp-stack" }
                        p { "A cross-platform Rust application & graphics stack — engine, UI, platform layer, comms, and tooling." }
                        ul {
                            li { span { "Quartz" } span { class: "k", "2D engine" } }
                            li { span { "wgpu_canvas" } span { class: "k", "GPU canvas" } }
                            li { span { "prism · pelican_ui" } span { class: "k", "UI" } }
                            li { span { "maverick_os" } span { class: "k", "platform · BLE" } }
                            li { span { "air" } span { class: "k", "encrypted comms" } }
                        }
                    }
                    div { class: "col",
                        div { class: "cen", "Space Soup VR" }
                        div { class: "csub", "github.com/SpaceSoupVR" }
                        p { "A WGPU 3D engine for the Meta Quest — rendering, engine, editor, and on-headset app." }
                        ul {
                            li { span { "space_soup" } span { class: "k", "renderer" } }
                            li { span { "space_soup_engine" } span { class: "k", "engine" } }
                            li { span { "space_soup_editor" } span { class: "k", "editor" } }
                            li { span { "quest_app" } span { class: "k", "C++ / Quest" } }
                            li { span { "agate" } span { class: "k", "2D UI" } }
                        }
                    }
                }
            }

            footer {
                div { class: "label", span { "Elsewhere" } span { class: "rule" } }
                div { class: "foot-links",
                    a { href: "https://github.com/ElijahNCouch", "GitHub" }
                    a { href: "https://github.com/ramp-stack", "ramp-stack" }
                    a { href: "https://github.com/SpaceSoupVR", "Space Soup VR" }
                }
                div { class: "colophon", "Elijah Couch — built in Rust (this whole page is WebAssembly)." br {} "© 2026" }
            }
        }
    }
}
