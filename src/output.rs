use crate::checks::CheckResult;

use core::cmp;

const COLUMNS: usize = 4;
///`Dimension` implementation
pub struct TabledDimensions {
    dimensions: [usize; COLUMNS]
}

impl TabledDimensions {
    pub const HEADER: [&'static str; COLUMNS] = ["Destination", "Kind", "Status", "Message"];
    ///Default padding
    pub const PADDING: usize = 2;

    ///Calculates table dimensions using provided `header` and `data`
    pub fn compute(data: &[CheckResult]) -> Self {
        use tabled::grid::util::string::get_string_width;

        let mut dimensions = [
            get_string_width(Self::HEADER[0]),
            get_string_width(Self::HEADER[1]),
            get_string_width(Self::HEADER[2]),
            get_string_width(Self::HEADER[3]),
        ];
        //once populated with header's default width, proceed to iterate over data computing max for every column
        for data in data {
            dimensions[0] = cmp::max(dimensions[0], get_string_width(data.destination.host_port()));
            dimensions[1] = cmp::max(dimensions[1], get_string_width(data.kind.as_str()));
            dimensions[2] = cmp::max(dimensions[2], get_string_width(data.status));
            dimensions[3] = cmp::max(dimensions[3], get_string_width(&data.message));
        }

        Self {
            dimensions
        }
    }
}

impl tabled::grid::dimension::Dimension for TabledDimensions {
    #[inline(always)]
    fn get_width(&self, column: usize) -> usize {
        //tabled needs at least 2 bytes of padding
        self.dimensions.get(column).copied().unwrap_or_default().saturating_add(Self::PADDING)
    }

    #[inline(always)]
    fn get_height(&self, _: usize) -> usize {
        1
    }
}

pub fn tabled(data: &[CheckResult]) -> tabled::tables::CompactTable<impl tabled::grid::records::IntoRecords<Cell = &'_ str>, TabledDimensions> {
    let dimensions = TabledDimensions::compute(data);
    let data = data.iter().map(|result| [
        result.destination.host_port(),
        result.kind.as_str(),
        result.status,
        result.message.as_str(),
    ]);
    let data = [TabledDimensions::HEADER].into_iter().chain(data);

    tabled::tables::CompactTable::with_dimension(data, dimensions).columns(COLUMNS)
}
