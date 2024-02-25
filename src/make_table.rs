use std::{collections::HashMap, fmt::Display};

use comfy_table::{presets::NOTHING, ColumnConstraint::UpperBoundary, Row, Table, Width::Fixed};

pub fn make_table(
	headers: Vec<impl Display>,
	rows: Vec<Vec<String>>,
	col_max_widths: Option<HashMap<usize, u16>>,
) -> String {
	let mut table = Table::new();

	table
		.load_preset(NOTHING)
		.set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
		.set_header(&headers);

	let column = table.column_mut(0).expect("should find a column");
	column.set_padding((0, 1));
	let column = table
		.column_mut(&headers.len() - 1)
		.expect("should find a column");
	column.set_padding((1, 0));

	if let Some(widths) = col_max_widths {
		for (index, max_width) in widths {
			let column = table.column_mut(index).expect("should find a column");
			column.set_constraint(UpperBoundary(Fixed(max_width)));
		}
	}

	for row_data in rows {
		let mut row = Row::from(row_data);
		row.max_height(1);
		table.add_row(row);
	}

	table.to_string()
}
