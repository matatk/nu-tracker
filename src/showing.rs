pub fn showing(number: usize) -> String {
	if number >= 30 {
		String::from("Showing the top 30")
	} else {
		format!("Showing {number}")
	}
}
