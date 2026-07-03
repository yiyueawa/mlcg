use std::thread;

use mlcg_core::{Instruction, LowerContext, PartialLine, PartialProgram, PartialToken, Processor};

struct ThreadProcessor;

#[derive(Debug)]
struct Noop;

impl Instruction<ThreadProcessor> for Noop {
    fn lower(
        &self,
        _ctx: &mut LowerContext<ThreadProcessor>,
        out: &mut PartialProgram<ThreadProcessor>,
    ) -> Result<(), mlcg_core::LowerError> {
        out.push_line(PartialLine::new(vec![PartialToken::raw("noop")]));
        Ok(())
    }
}

#[test]
fn processor_accepts_pushes_from_multiple_threads() {
    let processor = Processor::<ThreadProcessor>::new();

    let mut handles = Vec::new();
    for _ in 0..4 {
        let processor = processor.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..25 {
                processor.push(Noop);
            }
        }));
    }

    for handle in handles {
        handle.join().expect("thread joins");
    }

    let output = processor.emit().expect("emit succeeds");
    assert_eq!(output.lines().count(), 100);
}
