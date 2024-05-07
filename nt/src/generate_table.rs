use std::{collections::HashMap, fmt::Display};

use comfy_table::{presets::NOTHING, ColumnConstraint::UpperBoundary, Row, Table, Width::Fixed};
use crossterm::style::Stylize;

// FIXME: If a column is truncated, it loses its colour
pub fn generate_table(
	headers: &[impl Display],
	rows: Vec<Vec<String>>,
	first_col_max_width: Option<(usize, u16)>,
	other_col_max_widths: Option<HashMap<usize, u16>>,
) -> String {
	let mut table = Table::new();

	table
		.load_preset(NOTHING)
		.set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
		.set_header(
			headers
				.iter()
				.map(|h| h.to_string().bold())
				.collect::<Vec<_>>(),
		);

	let column = table.column_mut(0).expect("should find a column");
	column.set_padding((0, 1));
	let column = table
		.column_mut(&headers.len() - 1)
		.expect("should find a column");
	column.set_padding((1, 0));

	for row_data in rows {
		let mut row = Row::from(row_data);
		row.max_height(1);
		table.add_row(row);
	}

	let natural_col_widths = table.column_max_content_widths();
	let natural_table_width = natural_col_widths.iter().sum();
	let term_width = crossterm::terminal::size()
		.expect("terminal to have a size")
		.0;

	if term_width >= natural_table_width {
		// Do nowt - table fits entirely
	} else {
		// Try constraining one column first (likely the title column)
		if let Some((col_index, max_width)) = first_col_max_width {
			let cur_col_width = natural_col_widths[col_index];
			if natural_table_width - (cur_col_width - max_width) <= term_width {
				let column = table.column_mut(col_index).expect("should find a column");
				column.set_constraint(UpperBoundary(Fixed(max_width)));
				return table.to_string();
			}
		}

		// Apply all given column width constraints
		if let Some(widths) = other_col_max_widths {
			for (index, max_width) in widths {
				let column = table.column_mut(index).expect("should find a column");
				column.set_constraint(UpperBoundary(Fixed(max_width)));
			}
		}
	}

	table.to_string()
}
