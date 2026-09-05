#[allow(warnings)]
mod bindings;

use bindings::Guest;

struct MarkdownComponent;

impl Guest for MarkdownComponent {
    fn render(source: String) -> String {
        let mut options = comrak::Options::default();
        options.render.r#unsafe = true;
        comrak::markdown_to_html(&source, &options)
    }
}

bindings::export!(MarkdownComponent with_types_in bindings);
