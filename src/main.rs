#![allow(non_snake_case)]
// use std::{sync::{Arc, Mutex}, time::Duration};

use std::{sync::{ Arc, RwLock}, time::Duration};

// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;
use dotenv::dotenv;
use fermi::{Atom, use_read, use_init_atom_root};
use sysinfo::{System, SystemExt, CpuExt};

static SYSTEM: Atom<Arc<RwLock<System>>> = Atom(|_| {
    let sys = Arc::new(RwLock::new(System::new()));
    sys.write().unwrap().refresh_all();
    sys
});

static REFRESH_MS: Atom<u64> = Atom(|_| {
    let str = std::env::var("REFRESH_MS").expect("must provide refresh ms");
    str.parse().unwrap_or(1000)
});

fn main() {
    dotenv().ok();
    // launch the dioxus app in a webview
    dioxus_desktop::launch(App);
}

// define a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
    use_init_atom_root(cx);

    cx.render(rsx! {
        link { rel: "stylesheet", href: "../dist/tailwind.css" }
        div { class: "min-h-screen bg-black text-white flex justify-center",
            div { class: "container flex flex-col gap-8",
                h1 { class: "text-xl py-4", "Hello, world!" }
                Cpus {}
            }
        }
    })
}

#[inline_props]
fn Cpus(cx: Scope) -> Element {
    let sys = use_read(cx, &SYSTEM);
    let sleep_ms = use_read(cx, &REFRESH_MS);

    let cpus = {
        let sys = sys.read().unwrap();
        let cpus = sys.cpus();
        cpus.into_iter().map(|cpu| cpu.cpu_usage().clone()).collect::<Vec<f32>>()
    };
    let cpu_state = use_state(cx, || cpus.clone());

    let sys_clone = sys.clone();
    let cpus_clone = cpu_state.clone();
    let sleep_clone = sleep_ms.clone();
    use_coroutine(cx, |_: UnboundedReceiver<()>| async move {
        loop {
            {sys_clone.write().unwrap().refresh_cpu()};
            let cpus = {
                let sys = sys_clone.read().unwrap();
                let cpus = sys.cpus();
                cpus.into_iter().map(|cpu| cpu.cpu_usage().clone()).collect::<Vec<f32>>()
            };
            cpus_clone.set(cpus);
            tokio::time::sleep(Duration::from_millis(sleep_clone)).await;
        }
    });
    cx.render(rsx! {
        div { class: "grid grid-cols-4 gap-4",
            cpus.into_iter().enumerate().map(|(i, cpu)|
                rsx!{ 
                    div { 
                        class: "bg-red-500 p-4 flex flex-col",
                        div {
                            class: "text-sm",
                            format!("cpu {}", i+1)
                        }
                        div {
                            class: "text-lg",
                            format!("{}%", cpu.to_string()) 
                        }
                    }
                })
        }
    })
}


