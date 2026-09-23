use super::FormatElement;

pub(super) fn print(elements: &[FormatElement<'_>]) -> String {
    let mut output = String::new();
    for element in elements {
        match element {
            FormatElement::Text(text) => output.push_str(text),
        }
    }
    output
}
