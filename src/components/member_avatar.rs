use dioxus::prelude::*;
use uuid::Uuid;

use crate::{api::file_url, icons::*};

#[component]
pub fn MemberAvatar(img_id: Option<Uuid>, size: usize) -> Element {
    let mut show_avatar = use_signal(|| img_id.is_none());
    
    if show_avatar() {
        rsx! {
            Icon {
                class: "rounded-box foreground bg-primary-content inset-ring-2 inset-ring-primary text-primary",
                size: size * 4,
                data: lucide::User,
                stroke_width: 4,
            }
        }
    } else {
        rsx! {
            img {
                class: "size-[var(--s)] rounded-box foreground object-cover",
                style: format!("--s: {}px", size * 4),
                src: file_url(img_id.unwrap_or_default()),
                onerror: move |_| show_avatar.set(true),
            }
        }
    }
}
