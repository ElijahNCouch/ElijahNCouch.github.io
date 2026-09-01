// Interactive demos, implemented in Rust via web-sys.
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

fn win() -> web_sys::Window {
    web_sys::window().expect("no window")
}
fn doc() -> web_sys::Document {
    win().document().expect("no document")
}
fn rand() -> f64 {
    js_sys::Math::random()
}

fn is_dark() -> bool {
    if let Some(root) = doc().document_element() {
        match root.get_attribute("data-theme").as_deref() {
            Some("dark") => return true,
            Some("light") => return false,
            _ => {}
        }
    }
    win()
        .match_media("(prefers-color-scheme:dark)")
        .ok()
        .flatten()
        .map(|m| m.matches())
        .unwrap_or(false)
}

struct Palette {
    accent: &'static str,
    muted: &'static str,
    line: &'static str,
}
fn pal() -> Palette {
    if is_dark() {
        Palette { accent: "#2997ff", muted: "#86868b", line: "#2a2a2c" }
    } else {
        Palette { accent: "#0071e3", muted: "#6e6e73", line: "#d2d2d7" }
    }
}

fn canvas_by_id(id: &str) -> Option<HtmlCanvasElement> {
    doc().get_element_by_id(id)?.dyn_into::<HtmlCanvasElement>().ok()
}

fn fit(canvas: &HtmlCanvasElement, css_h: f64) -> (CanvasRenderingContext2d, f64, f64) {
    let rect = canvas.get_bounding_client_rect();
    let dpr = win().device_pixel_ratio().min(2.0).max(1.0);
    let w = rect.width().max(1.0);
    canvas.set_width((w * dpr) as u32);
    canvas.set_height((css_h * dpr) as u32);
    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    let _ = ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
    (ctx, w, css_h)
}

// self-perpetuating requestAnimationFrame loop
fn start_raf<F: FnMut(f64) + 'static>(mut f: F) {
    let cb: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let cb2 = cb.clone();
    *cb2.borrow_mut() = Some(Closure::wrap(Box::new(move |t: f64| {
        f(t);
        let _ = win().request_animation_frame(
            cb.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
        );
    }) as Box<dyn FnMut(f64)>));
    let _ = win()
        .request_animation_frame(cb2.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    // keep alive for the life of the page
    std::mem::forget(cb2);
}

fn on_click<F: FnMut() + 'static>(id: &str, mut f: F) {
    if let Some(el) = doc().get_element_by_id(id) {
        let cb = Closure::wrap(Box::new(move |_e: web_sys::Event| f()) as Box<dyn FnMut(_)>);
        let _ = el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
        cb.forget();
    }
}

pub fn setup_all() {
    setup_theme();
    setup_ble();
    setup_sort();
    setup_encrypt();
}

// ---------------- theme toggle (pure web-sys, no Dioxus re-render) ----------------
fn setup_theme() {
    // apply saved theme
    if let Ok(Some(store)) = win().local_storage() {
        if let Ok(Some(t)) = store.get_item("theme") {
            if let Some(root) = doc().document_element() {
                let _ = root.set_attribute("data-theme", &t);
            }
        }
    }
    let set_label = || {
        if let Some(btn) = doc().get_element_by_id("toggle") {
            btn.set_text_content(Some(if is_dark() { "Light" } else { "Dark" }));
        }
    };
    set_label();
    if let Some(btn) = doc().get_element_by_id("toggle") {
        let cb = Closure::wrap(Box::new(move |_e: web_sys::Event| {
            let next = if is_dark() { "light" } else { "dark" };
            if let Some(root) = doc().document_element() {
                let _ = root.set_attribute("data-theme", next);
            }
            if let Ok(Some(store)) = win().local_storage() {
                let _ = store.set_item("theme", next);
            }
            if let Some(b) = doc().get_element_by_id("toggle") {
                b.set_text_content(Some(if next == "dark" { "Light" } else { "Dark" }));
            }
        }) as Box<dyn FnMut(_)>);
        let _ = btn.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
        cb.forget();
    }
}

// ---------------- BLE scanner ----------------
struct Dev {
    name: String,
    ang: f64,
    dist: f64,
    rssi: i32,
    found: bool,
    pulse: f64,
    uuid: String,
}
struct Ble {
    devices: Vec<Dev>,
    sweep: f64,
    scanning: bool,
    last_spawn: f64,
}

fn ble_uuid() -> String {
    let h = b"0123456789ABCDEF";
    let mut s = String::from("6E40");
    for _ in 0..4 {
        s.push(h[(rand() * 16.0) as usize] as char);
    }
    s.push_str("-B5A3-F393-E0A9-E50E24DCCA9E");
    s
}

fn ble_render_list(st: &Ble) {
    let list = match doc().get_element_by_id("bleList") {
        Some(l) => l,
        None => return,
    };
    let mut found: Vec<&Dev> = st.devices.iter().filter(|d| d.found).collect();
    if found.is_empty() {
        let msg = if st.scanning {
            "Scanning…"
        } else {
            "Press Scan to discover<br>nearby BLE devices"
        };
        list.set_inner_html(&format!("<div class=\"ble-empty\">{}</div>", msg));
        return;
    }
    found.sort_by(|a, b| b.rssi.cmp(&a.rssi));
    let mut html = String::new();
    for d in found {
        let pct = (((d.rssi + 90) as f64 / 60.0 * 100.0).round() as i32).clamp(4, 100);
        html.push_str(&format!(
            "<div class=\"dev\"><div class=\"row1\"><span class=\"nm\">{}</span><span class=\"rssi\">{} dBm</span></div><div class=\"bar\"><i style=\"width:{}%\"></i></div><div class=\"gatt\"><b>service</b> {}<br><b>Shared</b> Read · Write · Notify<br><b>Custom</b> Read</div></div>",
            d.name, d.rssi, pct, d.uuid
        ));
    }
    list.set_inner_html(&html);
}

fn setup_ble() {
    let canvas = match canvas_by_id("bleRadar") {
        Some(c) => c,
        None => return,
    };
    let names = [
        "iPhone", "MacBook Pro", "AirPods Pro", "Quest 3", "iPad", "Apple Watch", "HomePod",
        "Unknown",
    ];
    let state = Rc::new(RefCell::new(Ble {
        devices: Vec::new(),
        sweep: 0.0,
        scanning: false,
        last_spawn: 0.0,
    }));

    // Scan button
    {
        let st = state.clone();
        on_click("bleScan", move || {
            let mut b = st.borrow_mut();
            b.scanning = !b.scanning;
            let scanning = b.scanning;
            if let Some(btn) = doc().get_element_by_id("bleScan") {
                btn.set_text_content(Some(if scanning { "Stop" } else { "Scan" }));
                let cl = btn.class_list();
                if scanning {
                    let _ = cl.remove_1("pri");
                    let _ = cl.add_1("ghost");
                } else {
                    let _ = cl.add_1("pri");
                    let _ = cl.remove_1("ghost");
                }
            }
            let empty = !b.devices.iter().any(|d| d.found);
            drop(b);
            if empty {
                ble_render_list(&st.borrow());
            }
        });
    }
    // Clear button
    {
        let st = state.clone();
        on_click("bleClear", move || {
            st.borrow_mut().devices.clear();
            ble_render_list(&st.borrow());
        });
    }
    // list expand (event delegation)
    if let Some(list) = doc().get_element_by_id("bleList") {
        let cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
            if let Some(t) = e.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                if let Ok(Some(dev)) = t.closest(".dev") {
                    let cl = dev.class_list();
                    if cl.contains("open") {
                        let _ = cl.remove_1("open");
                    } else {
                        let _ = cl.add_1("open");
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);
        let _ = list.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
        cb.forget();
    }

    ble_render_list(&state.borrow());

    let st = state.clone();
    let names: Vec<String> = names.iter().map(|s| s.to_string()).collect();
    let mut geom = fit(&canvas, 250.0);
    start_raf(move |t| {
        // refit occasionally for responsiveness
        let rect = canvas.get_bounding_client_rect();
        if (rect.width() - geom.1).abs() > 1.0 {
            geom = fit(&canvas, 250.0);
        }
        let (ctx, w, h) = (&geom.0, geom.1, geom.2);
        let mut b = st.borrow_mut();
        if b.scanning {
            b.sweep += 0.045;
            if t - b.last_spawn > 700.0 + rand() * 700.0 && b.devices.len() < 6 {
                b.last_spawn = t;
                let dist = 0.28 + rand() * 0.62;
                let dev = Dev {
                    name: names[(rand() * names.len() as f64) as usize].clone(),
                    ang: rand() * std::f64::consts::PI * 2.0,
                    dist,
                    rssi: (-30.0 - dist * 60.0).round() as i32,
                    found: false,
                    pulse: 0.0,
                    uuid: ble_uuid(),
                };
                b.devices.push(dev);
            }
            let sweep = b.sweep;
            let mut changed = false;
            for d in b.devices.iter_mut() {
                if !d.found {
                    let mut da = (sweep - d.ang) % (std::f64::consts::PI * 2.0);
                    if da < 0.0 {
                        da += std::f64::consts::PI * 2.0;
                    }
                    if da < 0.12 {
                        d.found = true;
                        d.pulse = 1.0;
                        changed = true;
                    }
                }
            }
            if changed {
                let snapshot = &*b;
                ble_render_list(snapshot);
            }
        }
        // draw
        let p = pal();
        let (cx, cy) = (w / 2.0, h / 2.0);
        let r = (w.min(h)) / 2.0 - 14.0;
        ctx.clear_rect(0.0, 0.0, w, h);
        ctx.set_stroke_style_str(p.line);
        ctx.set_line_width(1.0);
        for i in 1..=3 {
            ctx.begin_path();
            let _ = ctx.arc(cx, cy, r * i as f64 / 3.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.stroke();
        }
        if b.scanning {
            ctx.set_global_alpha(0.9);
            ctx.set_stroke_style_str(p.accent);
            ctx.begin_path();
            ctx.move_to(cx, cy);
            ctx.line_to(cx + b.sweep.cos() * r, cy + b.sweep.sin() * r);
            ctx.stroke();
            ctx.set_global_alpha(1.0);
        }
        for d in b.devices.iter_mut() {
            if !d.found {
                continue;
            }
            let px = cx + d.ang.cos() * d.dist * r;
            let py = cy + d.ang.sin() * d.dist * r;
            if d.pulse > 0.0 {
                ctx.set_global_alpha(d.pulse * 0.4);
                ctx.set_fill_style_str(p.accent);
                ctx.begin_path();
                let _ = ctx.arc(px, py, 10.0 * (1.3 - d.pulse), 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();
                ctx.set_global_alpha(1.0);
                d.pulse -= 0.03;
            }
            ctx.set_fill_style_str(p.accent);
            ctx.begin_path();
            let _ = ctx.arc(px, py, 4.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();
        }
        ctx.set_fill_style_str(p.muted);
        ctx.begin_path();
        let _ = ctx.arc(cx, cy, 3.0, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();
    });
}

// ---------------- sorting ----------------
struct Sort {
    arr: Vec<i32>,
    hi: (i32, i32),
    frames: Vec<(Vec<i32>, i32, i32)>,
    idx: usize,
    playing: bool,
    sound: bool,
    audio: Option<web_sys::AudioContext>,
}
const SN: i32 = 44;

fn sort_shuffle(st: &mut Sort) {
    st.arr = (1..=SN).collect();
    for i in (1..SN as usize).rev() {
        let j = (rand() * (i as f64 + 1.0)) as usize;
        st.arr.swap(i, j);
    }
    st.hi = (-1, -1);
    st.playing = false;
    st.frames.clear();
    st.idx = 0;
}

fn sort_frames(base: &[i32], kind: &str) -> Vec<(Vec<i32>, i32, i32)> {
    let mut a = base.to_vec();
    let mut f: Vec<(Vec<i32>, i32, i32)> = Vec::new();
    let n = a.len();
    let snap = |a: &Vec<i32>, i: usize, j: usize, f: &mut Vec<(Vec<i32>, i32, i32)>| {
        f.push((a.clone(), i as i32, j as i32));
    };
    match kind {
        "insertion" => {
            for i in 1..n {
                let mut j = i;
                while j > 0 && a[j - 1] > a[j] {
                    snap(&a, j - 1, j, &mut f);
                    a.swap(j - 1, j);
                    snap(&a, j - 1, j, &mut f);
                    j -= 1;
                }
            }
        }
        "selection" => {
            for i in 0..n.saturating_sub(1) {
                let mut m = i;
                for j in i + 1..n {
                    snap(&a, m, j, &mut f);
                    if a[j] < a[m] {
                        m = j;
                    }
                }
                if m != i {
                    a.swap(i, m);
                    snap(&a, i, m, &mut f);
                }
            }
        }
        _ => {
            for i in 0..n.saturating_sub(1) {
                for j in 0..n - 1 - i {
                    snap(&a, j, j + 1, &mut f);
                    if a[j] > a[j + 1] {
                        a.swap(j, j + 1);
                        snap(&a, j, j + 1, &mut f);
                    }
                }
            }
        }
    }
    f
}

fn sort_draw(ctx: &CanvasRenderingContext2d, st: &Sort, w: f64, h: f64) {
    let p = pal();
    ctx.clear_rect(0.0, 0.0, w, h);
    let bw = w / SN as f64;
    for (i, v) in st.arr.iter().enumerate() {
        let bh = *v as f64 / SN as f64 * (h - 10.0);
        let active = i as i32 == st.hi.0 || i as i32 == st.hi.1;
        ctx.set_fill_style_str(if active { p.accent } else { p.muted });
        ctx.set_global_alpha(if active { 1.0 } else { 0.5 });
        ctx.fill_rect(i as f64 * bw + 1.0, h - bh, bw - 2.0, bh);
    }
    ctx.set_global_alpha(1.0);
}

fn sort_beep(st: &mut Sort, v: i32) {
    if !st.sound {
        return;
    }
    if st.audio.is_none() {
        st.audio = web_sys::AudioContext::new().ok();
    }
    if let Some(ac) = &st.audio {
        if let (Ok(osc), Ok(gain)) = (ac.create_oscillator(), ac.create_gain()) {
            osc.set_type(web_sys::OscillatorType::Sine);
            osc.frequency().set_value(180.0 + v as f32 / SN as f32 * 720.0);
            gain.gain().set_value(0.05);
            let _ = osc.connect_with_audio_node(&gain);
            let _ = gain.connect_with_audio_node(&ac.destination());
            let now = ac.current_time();
            let _ = osc.start();
            let _ = gain.gain().exponential_ramp_to_value_at_time(0.0001, now + 0.06);
            let _ = osc.stop_with_when(now + 0.07);
        }
    }
}

fn setup_sort() {
    let canvas = match canvas_by_id("sortCv") {
        Some(c) => c,
        None => return,
    };
    let state = Rc::new(RefCell::new(Sort {
        arr: Vec::new(),
        hi: (-1, -1),
        frames: Vec::new(),
        idx: 0,
        playing: false,
        sound: true,
        audio: None,
    }));
    sort_shuffle(&mut state.borrow_mut());

    // Sort
    {
        let st = state.clone();
        on_click("sortRun", move || {
            let mut s = st.borrow_mut();
            if s.playing {
                return;
            }
            if let Some(ac) = &s.audio {
                if ac.state() == web_sys::AudioContextState::Suspended {
                    let _ = ac.resume();
                }
            }
            let kind = doc()
                .get_element_by_id("sortAlgo")
                .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
                .map(|e| e.value())
                .unwrap_or_else(|| "bubble".into());
            s.frames = sort_frames(&s.arr, &kind);
            s.idx = 0;
            s.playing = true;
        });
    }
    // Shuffle
    {
        let st = state.clone();
        on_click("sortShuffle", move || {
            let mut s = st.borrow_mut();
            if !s.playing {
                sort_shuffle(&mut s);
            }
        });
    }
    // Sound toggle
    {
        let st = state.clone();
        on_click("sortSound", move || {
            let mut s = st.borrow_mut();
            s.sound = !s.sound;
            if let Some(btn) = doc().get_element_by_id("sortSound") {
                btn.set_text_content(Some(if s.sound { "Sound: on" } else { "Sound: off" }));
            }
        });
    }

    let st = state.clone();
    let mut geom = fit(&canvas, 210.0);
    start_raf(move |_t| {
        let rect = canvas.get_bounding_client_rect();
        if (rect.width() - geom.1).abs() > 1.0 {
            geom = fit(&canvas, 210.0);
        }
        let mut s = st.borrow_mut();
        if s.playing {
            let total = s.frames.len();
            let step = (total / 260).max(1);
            let mut last_v = 1;
            for _ in 0..step {
                if s.idx < total {
                    let fr = s.frames[s.idx].clone();
                    s.arr = fr.0;
                    s.hi = (fr.1, fr.2);
                    last_v = if fr.1 >= 0 && (fr.1 as usize) < s.arr.len() {
                        s.arr[fr.1 as usize]
                    } else {
                        1
                    };
                    s.idx += 1;
                }
            }
            if s.idx >= total {
                s.hi = (-1, -1);
                s.playing = false;
            } else {
                sort_beep(&mut s, last_v);
            }
        }
        sort_draw(&geom.0, &s, geom.1, geom.2);
    });
}

// ---------------- encrypt ----------------
fn enc_key() -> Vec<u8> {
    (0..32).map(|_| (rand() * 256.0) as u8).collect()
}
fn enc_cipher(text: &str, key: &[u8]) -> Vec<u8> {
    text.bytes()
        .enumerate()
        .map(|(i, c)| c ^ key[i % key.len()] ^ ((i as u32 * 37) & 0xff) as u8)
        .collect()
}
fn hex_byte(b: u8) -> String {
    format!("{:02x}", b)
}

fn setup_encrypt() {
    let input = match doc()
        .get_element_by_id("encIn")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
    {
        Some(i) => i,
        None => return,
    };
    if input.value().is_empty() {
        let _ = input.set_value("hello from the metal");
    }
    let key = Rc::new(RefCell::new(enc_key()));
    // step counter drives a short scramble animation
    let anim = Rc::new(RefCell::new(0usize));
    let done = Rc::new(RefCell::new(true));

    fn show_key(key: &[u8]) {
        if let Some(el) = doc().get_element_by_id("encKey") {
            el.set_text_content(Some(&key.iter().map(|b| hex_byte(*b)).collect::<String>()));
        }
    }
    fn render_now(input: &HtmlInputElement, key: &[u8], step: usize) {
        let target = enc_cipher(&input.value(), key);
        let h = b"0123456789abcdef";
        let out: Vec<String> = target
            .iter()
            .enumerate()
            .map(|(i, b)| {
                if i <= step {
                    hex_byte(*b)
                } else {
                    let a = h[(rand() * 16.0) as usize] as char;
                    let c = h[(rand() * 16.0) as usize] as char;
                    format!("{}{}", a, c)
                }
            })
            .collect();
        if let Some(el) = doc().get_element_by_id("encOut") {
            el.set_text_content(Some(&out.join(" ")));
        }
    }

    show_key(&key.borrow());

    // animation loop for scramble reveal
    {
        let input2 = input.clone();
        let key2 = key.clone();
        let anim2 = anim.clone();
        let done2 = done.clone();
        start_raf(move |_t| {
            if *done2.borrow() {
                return;
            }
            let step = *anim2.borrow();
            render_now(&input2, &key2.borrow(), step);
            let len = input2.value().len();
            if step >= len {
                *done2.borrow_mut() = true;
            } else {
                *anim2.borrow_mut() = step + 2;
            }
        });
    }

    let trigger = {
        let anim = anim.clone();
        let done = done.clone();
        move || {
            *anim.borrow_mut() = 0;
            *done.borrow_mut() = false;
        }
    };

    // input event
    {
        let trig = trigger.clone();
        let cb = Closure::wrap(Box::new(move |_e: web_sys::Event| trig()) as Box<dyn FnMut(_)>);
        let _ = input.add_event_listener_with_callback("input", cb.as_ref().unchecked_ref());
        cb.forget();
    }
    // rotate key
    {
        let key = key.clone();
        let trig = trigger.clone();
        on_click("encRotate", move || {
            *key.borrow_mut() = enc_key();
            show_key(&key.borrow());
            trig();
        });
    }

    // initial render
    trigger();
}
