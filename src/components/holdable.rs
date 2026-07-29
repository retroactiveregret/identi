use dioxus::prelude::*;

#[component]
pub fn Holdable(
    #[props(default = 1000)] duration: u64,
    onhold: EventHandler<()>,
    #[props(default)] onclick: EventHandler<()>,
    children: Element,
) -> Element {
    let mut timer = use_signal(|| None::<gloo_timers::callback::Timeout>);
    let mut held = use_signal(|| false);

    rsx! {
        div {
            onpointerdown: move |_| {
                let onhold = onhold.clone();
                let duration = duration;

                timer
                    .set(
                        Some(
                            gloo_timers::callback::Timeout::new(
                                duration as u32,
                                move || {
                                    held.set(true);
                                    onhold.call(());
                                },
                            ),
                        ),
                    );
            },

            onpointerup: move |_| {
                if let Some(timeout) = timer.write().take() {
                    timeout.cancel();
                    held.set(false);
                }
                if !held() {
                    onclick.call(());
                }
            },

            onpointerleave: move |_| {
                if let Some(timeout) = timer.write().take() {
                    timeout.cancel();
                    held.set(false);
                }
            },

            {children}
        }
    }
}
