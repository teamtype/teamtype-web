use crate::services::node_service::{NodeCommand, SecretAddress};
use dioxus::prelude::*;
use std::string::ToString;

#[component]
fn SimpleForm() -> Element {
    rsx! {
        fieldset {
            label {
                for: "join_code",
                "magic wormhole code:"
            }

            input {
                id: "join_code",
                name: "join_code",
                style: "min-width: 40em;"
            }
        }
    }
}

#[component]
fn AdvancedForm() -> Element {
    rsx! {
        fieldset {
            label {
                for: "peer_secret_address",
                "peer's secret address:"
            }

            // TODO: should this be type: password?
            input {
                id: "peer_secret_address",
                name: "peer_secret_address",
                style: "min-width: 40em;"
            }
        }
    }
}

#[derive(Default)]
struct SimpleFormData {
    join_code: String,
}

#[derive(Default)]
struct AdvancedFormData {
    peer_secret_address: String,
}

#[component]
pub fn ConnectionForm() -> Element {
    let node_service = use_coroutine_handle::<NodeCommand>();

    let mut form_error = use_signal(|| "".to_string());
    let mut mode = use_signal(|| "simple".to_string());
    let simple_form_data = use_signal(SimpleFormData::default);
    let advanced_form_data = use_signal(AdvancedFormData::default);

    let onsubmit = move |event: FormEvent| {
        event.stop_propagation();

        form_error.set("".to_string());

        match mode.read().as_str() {
            "simple" => {
                let join_code = simple_form_data.read().join_code.clone();
                node_service.send(NodeCommand::ConnectByJoinCode { join_code });
            }
            "advanced" => {
                let peer_secret_address = advanced_form_data.read().peer_secret_address.clone();
                match SecretAddress::from_string(peer_secret_address) {
                    Ok(secret_address) => node_service.send(NodeCommand::ConnectByAddress {
                        secret_address: Box::new(secret_address),
                    }),
                    Err(error) => {
                        form_error.set(format!("{error}"));
                    }
                }
            }
            _ => {
                panic!("Unexpected form mode: {mode}")
            }
        };
    };

    rsx! {
        section {
            h2 { "Bidirectional Connection" }

            "{form_error}"

            form {
                onsubmit,

                fieldset {
                    for value in ["simple", "advanced"] {
                        label {
                            input {
                                type: "radio",
                                name: "mode",
                                checked: *mode.read() == value,
                                oninput: move |_| mode.set(value.to_string()),
                                value: value
                            },
                            "{value}"
                        }
                    }
                }

                match mode.read().as_str() {
                    "simple" => rsx! {
                        SimpleForm { }
                    },
                    "advanced" => rsx! {
                        AdvancedForm { }
                    },
                    _ => {
                        panic!("Unexpected form mode: {mode}")
                    }
                }

                button {
                    type: "submit",
                    "connect"
                }
            }
        }
    }
}
