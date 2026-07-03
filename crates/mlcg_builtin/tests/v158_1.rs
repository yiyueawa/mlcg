use mlcg_builtin::latest::prelude::*;
use mlcg_core::Processor;

struct P;

#[test]
fn generated_v158_1_api_emits_mlog_program() {
    let processor = Processor::<P>::new();
    let x = processor.named("x");
    let y = processor.named("y");

    x.set(1);
    let sum = processor.op_add(x.clone(), y.clone());
    let inverted = processor.op_not(sum.clone());
    processor.op_add_into(inverted.clone(), sum, 2);
    processor.print("message");
    let read_value = processor.read("cell1", 0);
    processor.read_into(x.clone(), "cell1", 1);
    processor.print(read_value);
    let cell = processor.named("cell");
    let value_read = cell.read(2);
    let value_sum = x.op_add(y);
    cell.write(x.clone(), 3);
    let sensed = cell.sensor("@enabled");
    let located = processor.ulocate_building("@core", true, "@copper");
    processor.ulocate_building_into(located.clone(), "@core", true, "@copper");
    processor.ulocate_building_into(located.as_tuple(), "@core", true, "@copper");
    let located_tuple = located.clone().into_tuple();
    processor.ulocate_building_into(&located_tuple, "@core", true, "@copper");
    let radar_target = processor.uradar("enemy", "any", "any", "distance", true);
    processor.uradar_into(radar_target.clone(), "enemy", "any", "any", "health", false);
    processor.print(value_read);
    processor.print(value_sum);
    processor.print(sensed);
    processor.print(radar_target);
    processor.print(located.outX);
    processor.print(located.outY);
    processor.print(located.outFound);
    processor.print(located.outBuild);
    let done = processor.label();
    x.jump_equal(done.clone(), 1);
    processor.print("before_done");
    processor.place(done);
    processor.print("after_done");

    let output = processor.emit().expect("emit succeeds");

    assert_eq!(
        output,
        "set x 1\nop add __mlcg_0 x y\nop not __mlcg_1 __mlcg_0 0\nop add __mlcg_1 __mlcg_0 2\nprint message\nread __mlcg_2 cell1 0\nread x cell1 1\nprint __mlcg_2\nread __mlcg_3 cell 2\nop add __mlcg_4 x y\nwrite x cell 3\nsensor __mlcg_5 cell @enabled\nulocate building @core true @copper __mlcg_6 __mlcg_7 __mlcg_8 __mlcg_9\nulocate building @core true @copper __mlcg_6 __mlcg_7 __mlcg_8 __mlcg_9\nulocate building @core true @copper __mlcg_6 __mlcg_7 __mlcg_8 __mlcg_9\nulocate building @core true @copper __mlcg_6 __mlcg_7 __mlcg_8 __mlcg_9\nuradar enemy any any distance 0 true __mlcg_10\nuradar enemy any any health 0 false __mlcg_10\nprint __mlcg_3\nprint __mlcg_4\nprint __mlcg_5\nprint __mlcg_10\nprint __mlcg_6\nprint __mlcg_7\nprint __mlcg_8\nprint __mlcg_9\njump 28 equal x 1\nprint before_done\nprint after_done"
    );
}

#[test]
fn generated_v158_1_condition_arity_keeps_only_meaningful_operands() {
    let processor = Processor::<P>::new();
    let x = processor.named("x");
    let y = processor.named("y");

    let selected = x.select_equal(y, "then", "else");
    let always_selected = processor.select_always("then", "else");
    let done = processor.label();
    processor.jump_always(done.clone());
    processor.print(selected);
    processor.place(done);
    processor.print(always_selected);

    let output = processor.emit().expect("emit succeeds");

    assert_eq!(
        output,
        "select __mlcg_0 equal x y then else\nselect __mlcg_1 always 0 0 then else\njump 4 always 0 0\nprint __mlcg_0\nprint __mlcg_1"
    );
}
