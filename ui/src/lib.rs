use gloo_net::http::Request;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    let input = use_state(String::new);
    let output = use_state(String::new);

    let on_input_change = {
        let input = input.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                input.set(target.value());
            }
        })
    };

    let on_send = {
        let input = input.clone();
        let output = output.clone();
        Callback::from(move |_| {
            let input = input.clone();
            let output = output.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::json!({
                    "model": "qwen3-8b-instruct",
                    "messages": [
                        { "role": "user", "content": (*input).clone() }
                    ]
                });

                match Request::post("http://localhost:8080/v1/chat/completions")
                    .header("Content-Type", "application/json")
                    .json(&body)
                {
                    Ok(req) => match req.send().await {
                        Ok(resp) => {
                            if let Ok(text) = resp.text().await {
                                output.set(text);
                            } else {
                                output.set("Failed to read response text".into());
                            }
                        }
                        Err(e) => {
                            output.set(format!("Gateway request error: {e}"));
                        }
                    },
                    Err(e) => {
                        output.set(format!("Failed to build request: {e}"));
                    }
                }
            });
        })
    };

    html! {
        <div style="max-width: 800px; margin: 1rem auto; font-family: sans-serif;">
            <h1>{ "Rust AI Stack Demo UI" }</h1>
            <p>{ "This Yew/WASM UI talks to the Rust gateway at http://localhost:8080." }</p>
            <label for="prompt">{ "Prompt:" }</label>
            <textarea
                id="prompt"
                rows={5}
                style="width: 100%;"
                value={(*input).clone()}
                oninput={on_input_change}
            />
            <button onclick={on_send} style="margin-top: 0.5rem;">{ "Send to LLM" }</button>
            <h2>{ "Raw response:" }</h2>
            <pre style="background:#f0f0f0; padding:0.5rem; white-space:pre-wrap;">
                { (*output).clone() }
            </pre>
            <p>{ "TTS endpoint (/v1/audio/speech) is wired but not used in this minimal UI yet." }</p>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run() {
    yew::Renderer::<App>::new().render();
}
