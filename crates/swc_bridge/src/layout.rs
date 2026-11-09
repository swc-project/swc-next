use swc_next_formatter_core::{IndentStyle, LineEnding, PrintOptions};

pub(crate) fn layout_options(
    width_low: u8,
    width_high: u8,
    indent_width: u8,
    use_tabs: u8,
    line_ending: u8,
) -> Option<PrintOptions> {
    Some(PrintOptions {
        line_width: u16::from_le_bytes([width_low, width_high]),
        indent_width: indent_width.try_into().ok()?,
        indent_style: match use_tabs {
            0 => IndentStyle::Space,
            1 => IndentStyle::Tab,
            _ => return None,
        },
        line_ending: match line_ending {
            0 => LineEnding::Lf,
            1 => LineEnding::CrLf,
            2 => LineEnding::Cr,
            _ => return None,
        },
    })
}
