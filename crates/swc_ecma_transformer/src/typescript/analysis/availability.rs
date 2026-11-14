//! Initialization order and deferred execution regions, independent of source spans.

use std::ops::Range;

#[derive(Clone, Copy)]
struct RegionId(usize);

#[derive(Clone, Copy)]
pub(super) struct Initialization {
    order: usize,
    region: RegionId,
}

pub(super) struct Availability {
    regions: Vec<Range<usize>>,
}

impl Availability {
    pub(super) fn is_available(&self, definition: Initialization, usage: Initialization) -> bool {
        definition.order < usage.order || !self.regions[usage.region.0].contains(&definition.order)
    }
}

pub(super) struct AvailabilityBuilder {
    regions: Vec<Range<usize>>,
    next_order: usize,
    region: RegionId,
}

pub(super) struct DeferredRegion {
    region: RegionId,
    parent: RegionId,
}

impl Default for AvailabilityBuilder {
    fn default() -> Self {
        Self {
            regions: std::iter::once(0..0).collect(),
            next_order: 0,
            region: RegionId(0),
        }
    }
}

impl AvailabilityBuilder {
    pub(super) fn initialization(&mut self) -> Initialization {
        let initialization = Initialization {
            order: self.next_order,
            region: self.region,
        };
        self.next_order += 1;
        initialization
    }

    pub(super) fn enter_deferred(&mut self) -> DeferredRegion {
        let region = RegionId(self.regions.len());
        self.regions.push(self.next_order..self.next_order);
        let parent = std::mem::replace(&mut self.region, region);
        DeferredRegion { region, parent }
    }

    pub(super) fn exit_deferred(&mut self, frame: DeferredRegion) {
        self.regions[frame.region.0].end = self.next_order;
        self.region = frame.parent;
    }

    pub(super) fn finish(mut self) -> Availability {
        self.regions[0].end = self.next_order;
        Availability {
            regions: self.regions,
        }
    }
}
