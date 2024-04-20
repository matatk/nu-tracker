/// The origin of the issue
#[derive(Clone)]
pub enum OriginQuery {
	/// Us
	OurGroup,
	/// A different group
	OtherGroup,
	/// I don't mind
	Whatevs,
}

impl OriginQuery {
	/// Create a new origin query
	// FIXME: param names
	pub fn new(ours: bool, others: bool) -> Self {
		match (ours, others) {
			(true, false) => OriginQuery::OurGroup,
			(false, true) => OriginQuery::OtherGroup,
			(false, false) => OriginQuery::Whatevs,
			(true, true) => {
				unreachable!("Clap should stop both 'ours' and 'others' from being set at once");
			}
		}
	}
}
