use operational_transform::OperationSeq;
use similar::{TextDiff, ChangeTag};

pub trait OperationFactory {
	fn content(&self) -> String;

	fn replace(&self, txt: &str) -> Option<OperationSeq> {
		self.delta(0, txt, self.content().len())
	}

	fn delta(&self, start: usize, txt: &str, end: usize) -> Option<OperationSeq> {
		let mut out = OperationSeq::default();
		let content = self.content();
		let tail_skip = content.len() - end;
		let content_slice = &content[start..tail_skip];

		if content_slice == txt {
			// if slice equals given text, no operation should be taken
			return None;
		}

		out.retain(start as u64);

		let diff = TextDiff::from_chars(content_slice, txt);

		for change in diff.iter_all_changes() {
			match change.tag() {
				ChangeTag::Equal => out.retain(1),
				ChangeTag::Delete => out.delete(1),
				ChangeTag::Insert => out.insert(change.value()),
			}
		}

		out.retain(tail_skip as u64);

		Some(out)
	}

	fn insert(&self, txt: &str, pos: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let total = self.content().len() as u64;
		out.retain(pos);
		out.insert(txt);
		out.retain(total - pos);
		out
	}

	fn delete(&self, pos: u64, count: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let len = self.content().len() as u64;
		out.retain(pos - count);
		out.delete(count);
		out.retain(len - pos);
		out
	}

	fn cancel(&self, pos: u64, count: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let len = self.content().len() as u64;
		out.retain(pos);
		out.delete(count);
		out.retain(len - (pos+count));
		out
	}
}
