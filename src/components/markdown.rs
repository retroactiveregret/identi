use dioxus::prelude::*;

#[component]
pub fn Markdown(
    text: String,
    #[props(default = true)] sanitize: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let parser_options = pulldown_cmark::Options::all();
    let text = hard_breaks(&text);
    let parser = pulldown_cmark::Parser::new_ext(&text, parser_options);

    let mut unsafe_html_output = String::new();
    pulldown_cmark::html::push_html(&mut unsafe_html_output, parser);

    let safe_html = if sanitize {
        ammonia::clean(&*unsafe_html_output)
    } else {
        unsafe_html_output
    };

    rsx! {
        article { dangerous_inner_html: safe_html, ..attributes }
    }
}

fn hard_breaks(input: &str) -> String {
    let mut out = String::new();
    let mut lines = input.lines().peekable();

    while let Some(line) = lines.next() {
        out.push_str(line);

        if let Some(next) = lines.peek() {
            if line.is_empty() || next.is_empty() {
                out.push('\n');
            } else {
                out.push_str("  \n");
            }
        }
    }

    out
}