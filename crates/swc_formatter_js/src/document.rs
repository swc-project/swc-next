//! Language-specific compositions of the shared document primitives.

use std::convert::Infallible;

use swc_next_formatter_core::FillSeparatorPolicy;

pub(crate) use swc_next_formatter_core::{
    ConditionTarget, FillEntry, FlatFragment, Fragment, GroupId, GroupRequest, NeutralFragment,
    RecordedGroup, Writer,
};

pub(crate) trait WriterExt<'a> {
    fn join_fragments(
        &mut self,
        left: Fragment<'a>,
        separate: bool,
        right: Fragment<'a>,
    ) -> Fragment<'a>;

    fn conditional_group_fragment(
        &mut self,
        prefix: Fragment<'a>,
        request: GroupRequest,
        open: &'a str,
        body: Fragment<'a>,
        close: &'a str,
    ) -> Fragment<'a>;

    fn flat_delimited_fragment(
        &mut self,
        open: &'a str,
        prefix: FlatFragment<'a>,
        body: Fragment<'a>,
        suffix: FlatFragment<'a>,
        close: &'a str,
    ) -> Fragment<'a>;

    fn indented_delimited_fragment(
        &mut self,
        open: &'a str,
        prefix: Fragment<'a>,
        body: Fragment<'a>,
        suffix: Fragment<'a>,
        close: &'a str,
    ) -> Fragment<'a>;

    fn fill_with_empty_line_policy<T, E>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<(T, bool), E>,
    ) -> Result<T, E>;
}

fn record<'a>(writer: &mut Writer<'a>, write: impl FnOnce(&mut Writer<'a>)) -> Fragment<'a> {
    let (fragment, ()) = writer
        .record_with_value(|writer| {
            write(writer);
            Ok::<_, Infallible>(())
        })
        .unwrap_or_else(|never| match never {});
    fragment
}

impl<'a> WriterExt<'a> for Writer<'a> {
    fn join_fragments(
        &mut self,
        left: Fragment<'a>,
        separate: bool,
        right: Fragment<'a>,
    ) -> Fragment<'a> {
        record(self, |writer| {
            writer.write_fragment(left);
            if separate {
                writer.space();
            }
            writer.write_fragment(right);
        })
    }

    fn conditional_group_fragment(
        &mut self,
        prefix: Fragment<'a>,
        request: GroupRequest,
        open: &'a str,
        body: Fragment<'a>,
        close: &'a str,
    ) -> Fragment<'a> {
        record(self, |writer| {
            writer.write_fragment(prefix);
            writer.group_with_request(request, |writer| {
                writer.if_break(open);
                writer.indent(|writer| {
                    writer.soft_line();
                    writer.write_fragment(body);
                });
                writer.soft_line();
                writer.if_break(close);
            });
        })
    }

    fn flat_delimited_fragment(
        &mut self,
        open: &'a str,
        prefix: FlatFragment<'a>,
        body: Fragment<'a>,
        suffix: FlatFragment<'a>,
        close: &'a str,
    ) -> Fragment<'a> {
        record(self, |writer| {
            writer.text(open);
            writer.write_flat_fragment(prefix);
            writer.write_fragment(body);
            writer.write_flat_fragment(suffix);
            writer.text(close);
        })
    }

    fn indented_delimited_fragment(
        &mut self,
        open: &'a str,
        prefix: Fragment<'a>,
        body: Fragment<'a>,
        suffix: Fragment<'a>,
        close: &'a str,
    ) -> Fragment<'a> {
        record(self, |writer| {
            writer.group(false, |writer| {
                writer.text(open);
                writer.indent(|writer| {
                    writer.write_fragment(prefix);
                    writer.write_fragment(body);
                });
                writer.write_fragment(suffix);
                writer.text(close);
            });
        })
    }

    fn fill_with_empty_line_policy<T, E>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<(T, bool), E>,
    ) -> Result<T, E> {
        self.fill_with_separator_policy(|writer| match write(writer) {
            Ok((value, true)) => (Ok(value), FillSeparatorPolicy::CollapseEmptyLines),
            Ok((value, false)) => (Ok(value), FillSeparatorPolicy::Preserve),
            Err(error) => (Err(error), FillSeparatorPolicy::Preserve),
        })
    }
}

#[cfg(test)]
mod tests;
