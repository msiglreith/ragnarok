use crate::{Device, Error};

pub struct TimerQueries(pub(crate) d3d12::QueryHeap);

impl Device {
    pub fn create_timer_queries(&self, num: usize) -> Result<TimerQueries, Error> {
        let (heap, _) = self.create_query_heap(d3d12::QueryHeapType::Timestamp, num as _, 0);
        Ok(TimerQueries(heap))
    }
}
